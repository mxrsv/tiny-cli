//! Quit (SIGTERM) or force quit (SIGKILL) one process. The target is
//! revalidated against a fresh lookup immediately before signalling, and every
//! result is a `TerminateOutcome` value rather than an error. There is no
//! automatic escalation, no tree signal and no elevation.

use std::fmt;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, System, UpdateKind};

use super::snapshot::{raw_process, to_info, ProcessInfo};
use crate::error::{Error, Result};

/// Refused even for the current user: quitting these breaks the login session.
pub const PROTECTED_NAMES: [&str; 5] = [
    "WindowServer",
    "loginwindow",
    "Dock",
    "SystemUIServer",
    "Finder",
];
/// How long to wait for the target to exit after the signal.
pub const SETTLE_TIMEOUT: Duration = Duration::from_secs(2);
const SETTLE_POLL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminateKind {
    /// SIGTERM: the process may clean up or ignore it.
    Graceful,
    /// SIGKILL: confirmed separately by the caller; never sent automatically.
    Force,
}

impl TerminateKind {
    fn signal(self) -> libc::c_int {
        match self {
            Self::Graceful => libc::SIGTERM,
            Self::Force => libc::SIGKILL,
        }
    }
}

/// The process as the caller saw it when the user confirmed the action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminateTarget {
    pub pid: u32,
    /// Seconds since the Unix epoch, as in `ProcessInfo::start_time`.
    pub start_time: u64,
    pub name: String,
    /// Compared only when the caller knew it; pass `None` when unreadable.
    pub exe: Option<PathBuf>,
}

impl From<&ProcessInfo> for TerminateTarget {
    fn from(info: &ProcessInfo) -> Self {
        Self {
            pid: info.pid,
            start_time: info.start_time,
            name: info.name.clone(),
            exe: info.exe.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "camelCase")]
pub enum Refusal {
    OwnProcess,
    OwnParent,
    /// PID 0 (kernel), PID 1 (launchd), or a PID outside the signalable range.
    SystemProcess,
    OtherUser,
    Protected {
        name: String,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnProcess => f.write_str("it is this process"),
            Self::OwnParent => f.write_str("it is the parent of this process"),
            Self::SystemProcess => f.write_str("it is a system process (PID 0 or 1)"),
            Self::OtherUser => f.write_str("it belongs to another user"),
            Self::Protected { name } => write!(f, "{name} is protected"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminateOutcome {
    /// The signal was delivered and the process has exited.
    Exited,
    /// The signal was delivered but the process was still running after
    /// `SETTLE_TIMEOUT`.
    StillRunning,
    /// No live process had this PID before the signal (ESRCH, gone, or zombie).
    AlreadyExited,
    /// macOS rejected the signal (EPERM).
    PermissionDenied,
    /// The PID now belongs to another process: start time, name or path changed.
    IdentityChanged,
    Refused(Refusal),
}

/// Why the current user may not act on this process, if anything. Shared by
/// `terminate` and port owner lookups.
pub fn refusal(info: &ProcessInfo) -> Option<Refusal> {
    pid_refusal(info.pid).or_else(|| {
        if !info.is_current_user {
            Some(Refusal::OtherUser)
        } else {
            protected_name(info).map(|name| Refusal::Protected { name })
        }
    })
}

fn pid_refusal(pid: u32) -> Option<Refusal> {
    if signalable_pid(pid).is_none() {
        Some(Refusal::SystemProcess)
    } else if pid == std::process::id() {
        Some(Refusal::OwnProcess)
    } else if pid == std::os::unix::process::parent_id() {
        Some(Refusal::OwnParent)
    } else {
        None
    }
}

/// Only PIDs above 1 address exactly one process: 0, -1 and negative values
/// signal process groups or every process the user owns.
fn signalable_pid(pid: u32) -> Option<libc::pid_t> {
    libc::pid_t::try_from(pid).ok().filter(|&pid| pid > 1)
}

fn protected_name(info: &ProcessInfo) -> Option<String> {
    let exe_name = info.exe.as_deref().and_then(|exe| exe.file_name());
    PROTECTED_NAMES
        .iter()
        .find(|&&name| info.name == name || exe_name.is_some_and(|exe| exe == name))
        .map(|name| name.to_string())
}

pub fn terminate(target: &TerminateTarget, kind: TerminateKind) -> Result<TerminateOutcome> {
    if let Some(refusal) = pid_refusal(target.pid) {
        return Ok(TerminateOutcome::Refused(refusal));
    }
    if let Some(outcome) = revalidate(target) {
        return Ok(outcome);
    }
    match send_signal(target.pid, kind)? {
        Some(outcome) => Ok(outcome),
        None => Ok(settle(target)),
    }
}

/// `None` when the live process still matches the target and may be signalled.
fn revalidate(target: &TerminateTarget) -> Option<TerminateOutcome> {
    let Some(live) = probe(target.pid) else {
        return Some(TerminateOutcome::AlreadyExited);
    };
    let exe_changed = target.exe.is_some() && target.exe != live.exe;
    if live.start_time != target.start_time || live.name != target.name || exe_changed {
        return Some(TerminateOutcome::IdentityChanged);
    }
    refusal(&live).map(TerminateOutcome::Refused)
}

/// `Ok(None)` when the signal was delivered.
fn send_signal(pid: u32, kind: TerminateKind) -> Result<Option<TerminateOutcome>> {
    let Some(pid) = signalable_pid(pid) else {
        return Ok(Some(TerminateOutcome::Refused(Refusal::SystemProcess)));
    };
    // SAFETY: kill has no memory-safety preconditions; pid > 1 addresses one process.
    if unsafe { libc::kill(pid, kind.signal()) } == 0 {
        return Ok(None);
    }
    let error = std::io::Error::last_os_error();
    match error.raw_os_error() {
        Some(libc::ESRCH) => Ok(Some(TerminateOutcome::AlreadyExited)),
        Some(libc::EPERM) => Ok(Some(TerminateOutcome::PermissionDenied)),
        _ => Err(Error::Operation(format!("kill({pid}) failed: {error}"))),
    }
}

fn settle(target: &TerminateTarget) -> TerminateOutcome {
    let deadline = Instant::now() + SETTLE_TIMEOUT;
    loop {
        let alive = probe(target.pid).is_some_and(|live| live.start_time == target.start_time);
        if !alive {
            return TerminateOutcome::Exited;
        }
        if Instant::now() >= deadline {
            return TerminateOutcome::StillRunning;
        }
        thread::sleep(SETTLE_POLL);
    }
}

/// Fresh lookup of one PID in its own `System`, so a shared `Sampler`'s CPU
/// baseline is untouched. A zombie has exited and counts as gone. A process
/// whose effective UID differs (for example `sudo`) is not the current user's.
fn probe(pid: u32) -> Option<ProcessInfo> {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::new()
            .with_user(UpdateKind::Always)
            .with_exe(UpdateKind::Always),
    );
    let process = system
        .process(pid)
        .filter(|process| process.status() != ProcessStatus::Zombie)?;
    // SAFETY: getuid has no preconditions and cannot fail.
    let current_uid = unsafe { libc::getuid() };
    let effective_uid = process.effective_user_id().map(|uid| **uid);
    let mut info = to_info(raw_process(process, None), false, current_uid);
    info.is_current_user &= effective_uid == Some(current_uid);
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command};

    fn info(pid: u32, name: &str, mine: bool) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.into(),
            user: None,
            uid: None,
            is_current_user: mine,
            parent_pid: None,
            start_time: 0,
            cpu_percent: None,
            cpu_measured: false,
            memory_bytes: None,
            exe: None,
        }
    }

    /// A disposable child: the only kind of process these tests signal.
    fn spawn_sleeper() -> (Child, TerminateTarget) {
        let child = Command::new("/bin/sleep").arg("30").spawn().unwrap();
        let live = probe(child.id()).expect("child is visible");
        (child, TerminateTarget::from(&live))
    }

    #[test]
    /// Pure checks only: `send_signal` is never called with these PIDs.
    fn unsignalable_pids_are_rejected_by_the_guard() {
        for pid in [0, 1, u32::MAX, i32::MAX as u32 + 1] {
            assert_eq!(signalable_pid(pid), None, "pid {pid}");
            assert_eq!(pid_refusal(pid), Some(Refusal::SystemProcess), "pid {pid}");
        }
        assert_eq!(signalable_pid(2), Some(2));
        assert_eq!(signalable_pid(i32::MAX as u32), Some(i32::MAX));
    }

    #[test]
    fn refuses_self_parent_system_other_user_and_protected() {
        let me = std::process::id();
        let parent = std::os::unix::process::parent_id();
        assert_eq!(refusal(&info(me, "x", true)), Some(Refusal::OwnProcess));
        assert_eq!(refusal(&info(parent, "x", true)), Some(Refusal::OwnParent));
        assert_eq!(
            refusal(&info(1, "launchd", true)),
            Some(Refusal::SystemProcess)
        );
        assert_eq!(
            refusal(&info(0, "kernel_task", true)),
            Some(Refusal::SystemProcess)
        );
        assert_eq!(refusal(&info(4242, "x", false)), Some(Refusal::OtherUser));
        assert_eq!(
            refusal(&info(4242, "Dock", true)),
            Some(Refusal::Protected {
                name: "Dock".into()
            })
        );
        let mut by_path = info(4242, "renamed", true);
        by_path.exe = Some("/System/Library/CoreServices/Finder.app/Contents/MacOS/Finder".into());
        assert!(matches!(refusal(&by_path), Some(Refusal::Protected { .. })));
        assert_eq!(refusal(&info(4242, "sleep", true)), None);
    }

    #[test]
    fn launchd_is_refused_before_any_lookup() {
        let target = TerminateTarget {
            pid: 1,
            start_time: 0,
            name: "launchd".into(),
            exe: None,
        };
        let outcome = terminate(&target, TerminateKind::Graceful).unwrap();
        assert_eq!(outcome, TerminateOutcome::Refused(Refusal::SystemProcess));
    }

    #[test]
    fn graceful_quit_reports_exit_of_an_unreaped_child() {
        let (mut child, target) = spawn_sleeper();
        let outcome = terminate(&target, TerminateKind::Graceful);
        // The child stays a zombie until this wait, so `Exited` proves zombies count as gone.
        child.kill().ok();
        let status = child.wait().unwrap();
        assert_eq!(outcome.unwrap(), TerminateOutcome::Exited);
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(libc::SIGTERM));
    }

    #[test]
    fn force_quit_sends_sigkill() {
        let (mut child, target) = spawn_sleeper();
        let outcome = terminate(&target, TerminateKind::Force);
        child.kill().ok();
        let status = child.wait().unwrap();
        assert_eq!(outcome.unwrap(), TerminateOutcome::Exited);
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(libc::SIGKILL));
    }

    #[test]
    fn changed_identity_is_not_signalled() {
        let (mut child, target) = spawn_sleeper();
        let stale_start = TerminateTarget {
            start_time: target.start_time - 1,
            ..target.clone()
        };
        let renamed = TerminateTarget {
            name: "other".into(),
            ..target.clone()
        };
        let moved = TerminateTarget {
            exe: Some("/bin/other".into()),
            ..target.clone()
        };
        let unknown_exe = TerminateTarget {
            exe: None,
            ..target.clone()
        };
        let outcomes = [&stale_start, &renamed, &moved].map(revalidate);
        let alive = child.try_wait().unwrap().is_none();
        let unknown_exe_ok = revalidate(&unknown_exe);
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(alive);
        assert!(outcomes
            .iter()
            .all(|o| *o == Some(TerminateOutcome::IdentityChanged)));
        assert_eq!(unknown_exe_ok, None);
    }

    #[test]
    fn exited_child_is_already_exited() {
        let (mut child, target) = spawn_sleeper();
        child.kill().unwrap();
        child.wait().unwrap();
        let outcome = terminate(&target, TerminateKind::Graceful).unwrap();
        assert_eq!(outcome, TerminateOutcome::AlreadyExited);
    }

    #[test]
    fn child_ignoring_sigterm_is_still_running() {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "trap '' TERM; exec /bin/sleep 30"])
            .spawn()
            .unwrap();
        // Wait for the exec so the target identity is the final `sleep`.
        let deadline = Instant::now() + Duration::from_secs(3);
        let live = loop {
            let live = probe(child.id()).expect("child is visible");
            if live.name == "sleep" || Instant::now() >= deadline {
                break live;
            }
            thread::sleep(SETTLE_POLL);
        };
        let outcome = terminate(&TerminateTarget::from(&live), TerminateKind::Graceful);
        child.kill().unwrap();
        child.wait().unwrap();
        assert_eq!(outcome.unwrap(), TerminateOutcome::StillRunning);
    }
}
