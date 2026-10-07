//! Process sampling. Missing measurements stay explicit (`None`) instead of
//! being reported as freshly measured zeroes.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: Option<String>,
    #[serde(skip)]
    pub uid: Option<u32>,
    pub is_current_user: bool,
    pub parent_pid: Option<u32>,
    /// Seconds since the Unix epoch, 1 s granularity.
    pub start_time: u64,
    /// Per-core percentage as reported by `sysinfo`; can exceed 100 on
    /// multi-core machines. `None` until measured or when access is denied.
    pub cpu_percent: Option<f32>,
    pub cpu_measured: bool,
    /// Resident memory. `None` when the process cannot be inspected.
    pub memory_bytes: Option<u64>,
    #[serde(skip)]
    pub exe: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSnapshot {
    pub processes: Vec<ProcessInfo>,
    /// Seconds since the Unix epoch.
    pub sampled_at: u64,
    /// True once every process present since the first sample has measured
    /// CPU. Processes that appeared later report their own `cpu_measured`.
    pub cpu_measured: bool,
}

/// Raw values for one process, before availability rules are applied.
pub(crate) struct RawProcess {
    pub pid: u32,
    pub name: String,
    pub uid: Option<u32>,
    pub user: Option<String>,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
    pub cpu: f32,
    pub memory: u64,
    pub exe: Option<PathBuf>,
}

/// Applies the availability rules. A live process never has zero resident
/// memory, so `memory == 0` means macOS denied task info (another user's
/// process); CPU from the same call is then unavailable too.
pub(crate) fn to_info(raw: RawProcess, cpu_measured: bool, current_uid: u32) -> ProcessInfo {
    let readable = raw.memory > 0;
    let measured = cpu_measured && readable;
    ProcessInfo {
        pid: raw.pid,
        name: raw.name,
        user: raw.user,
        is_current_user: raw.uid == Some(current_uid),
        uid: raw.uid,
        parent_pid: raw.parent_pid,
        start_time: raw.start_time,
        cpu_percent: measured.then_some(raw.cpu),
        cpu_measured: measured,
        memory_bytes: readable.then_some(raw.memory),
        exe: raw.exe,
    }
}

/// `sysinfo` on macOS records a new process's CPU times on its second refresh
/// and reports a real value only from the third; earlier values read 0.
const REFRESHES_FOR_CPU: u32 = 3;

/// Keeps one `System` alive so later samples have measured CPU.
pub struct Sampler {
    system: System,
    users: Users,
    refreshes: u32,
    /// pid -> (start time, refreshes this process has been seen in).
    seen: HashMap<u32, (u64, u32)>,
    current_uid: u32,
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            users: Users::new_with_refreshed_list(),
            refreshes: 0,
            seen: HashMap::new(),
            // SAFETY: getuid has no preconditions and cannot fail.
            current_uid: unsafe { libc::getuid() },
        }
    }

    pub fn current_uid(&self) -> u32 {
        self.current_uid
    }

    /// Refreshes once. A process has measured CPU from its third sample on.
    pub fn sample(&mut self) -> ProcessSnapshot {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new()
                .with_cpu()
                .with_memory()
                .with_user(UpdateKind::OnlyIfNotSet)
                .with_exe(UpdateKind::OnlyIfNotSet),
        );
        self.refreshes = self.refreshes.saturating_add(1);
        self.track_seen();
        self.snapshot()
    }

    fn track_seen(&mut self) {
        let mut seen = HashMap::with_capacity(self.system.processes().len());
        for process in self.system.processes().values() {
            let pid = process.pid().as_u32();
            let start = process.start_time();
            let count = match self.seen.get(&pid) {
                Some(&(previous_start, count)) if previous_start == start => count + 1,
                _ => 1,
            };
            seen.insert(pid, (start, count));
        }
        self.seen = seen;
    }

    /// For one-shot callers such as the CLI: refreshes until CPU is measured,
    /// waiting the minimum CPU interval between refreshes (about 0.4 s).
    pub fn sample_measured(&mut self) -> ProcessSnapshot {
        while self.refreshes + 1 < REFRESHES_FOR_CPU {
            self.sample();
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        }
        self.sample()
    }

    fn snapshot(&self) -> ProcessSnapshot {
        let processes = self
            .system
            .processes()
            .values()
            .map(|process| {
                let uid = process.user_id().map(|uid| **uid);
                let raw = RawProcess {
                    pid: process.pid().as_u32(),
                    name: process.name().to_string_lossy().into_owned(),
                    user: process
                        .user_id()
                        .and_then(|uid| self.users.get_user_by_id(uid))
                        .map(|user| user.name().to_string()),
                    uid,
                    parent_pid: process.parent().map(|pid| pid.as_u32()),
                    start_time: process.start_time(),
                    cpu: process.cpu_usage(),
                    memory: process.memory(),
                    exe: process.exe().map(PathBuf::from),
                };
                let seen = self.seen.get(&raw.pid).map_or(0, |&(_, count)| count);
                to_info(raw, seen >= REFRESHES_FOR_CPU, self.current_uid)
            })
            .collect();
        ProcessSnapshot {
            processes,
            sampled_at: unix_now(),
            cpu_measured: self.refreshes >= REFRESHES_FOR_CPU,
        }
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(memory: u64, uid: u32) -> RawProcess {
        RawProcess {
            pid: 42,
            name: "sample".into(),
            uid: Some(uid),
            user: Some("me".into()),
            parent_pid: Some(1),
            start_time: 1_700_000_000,
            cpu: 12.5,
            memory,
            exe: None,
        }
    }

    #[test]
    fn first_sample_has_no_cpu_value() {
        let info = to_info(raw(4096, 501), false, 501);
        assert_eq!(info.cpu_percent, None);
        assert!(!info.cpu_measured);
        assert_eq!(info.memory_bytes, Some(4096));
        assert!(info.is_current_user);
    }

    #[test]
    fn unreadable_process_reports_unavailable_not_zero() {
        let info = to_info(raw(0, 0), true, 501);
        assert_eq!(info.memory_bytes, None);
        assert_eq!(info.cpu_percent, None);
        assert!(!info.cpu_measured);
        assert!(!info.is_current_user);
    }

    #[test]
    fn measured_sample_keeps_cpu_value() {
        let info = to_info(raw(4096, 501), true, 501);
        assert_eq!(info.cpu_percent, Some(12.5));
        assert!(info.cpu_measured);
    }
}
