//! Process sampling. Missing measurements stay explicit (`None`) instead of
//! being reported as freshly measured zeroes.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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
    /// Native overview only; preserve the existing CLI JSON contract.
    #[serde(skip)]
    pub system_usage: SystemUsage,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemUsage {
    /// Whole-machine utilization, unlike per-core process percentages.
    pub cpu_percent: Option<f32>,
    pub cpu_warming_up: bool,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
}

impl SystemUsage {
    fn from_readings(cpu: f32, measured: bool, total: u64, used: u64) -> Self {
        let memory_available = total > 0 && used <= total;
        Self {
            cpu_percent: (measured && cpu.is_finite()).then_some(cpu.clamp(0.0, 100.0)),
            cpu_warming_up: !measured,
            memory_used_bytes: memory_available.then_some(used),
            memory_total_bytes: memory_available.then_some(total),
        }
    }
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

pub(crate) fn raw_process(process: &sysinfo::Process, user: Option<String>) -> RawProcess {
    RawProcess {
        pid: process.pid().as_u32(),
        name: process.name().to_string_lossy().into_owned(),
        user,
        uid: process.user_id().map(|uid| **uid),
        parent_pid: process.parent().map(|pid| pid.as_u32()),
        start_time: process.start_time(),
        cpu: process.cpu_usage(),
        memory: process.memory(),
        exe: process.exe().map(PathBuf::from),
    }
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
    system_cpu_samples: u32,
    last_system_cpu_sample: Option<Instant>,
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
            system_cpu_samples: 0,
            last_system_cpu_sample: None,
            // SAFETY: getuid has no preconditions and cannot fail.
            current_uid: unsafe { libc::getuid() },
        }
    }

    pub fn current_uid(&self) -> u32 {
        self.current_uid
    }

    /// Refreshes once. A process has measured CPU from its third sample on.
    pub fn sample(&mut self) -> ProcessSnapshot {
        self.refresh_system_usage();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new()
                .with_cpu()
                .with_memory()
                .with_user(UpdateKind::OnlyIfNotSet)
                .with_exe(UpdateKind::Always),
        );
        self.refreshes = self.refreshes.saturating_add(1);
        self.track_seen();
        self.snapshot()
    }

    fn refresh_system_usage(&mut self) {
        self.system.refresh_memory();
        let now = Instant::now();
        let due = self.last_system_cpu_sample.is_none_or(|previous| {
            now.duration_since(previous) >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
        });
        if due {
            self.system.refresh_cpu_usage();
            self.system_cpu_samples = self.system_cpu_samples.saturating_add(1);
            self.last_system_cpu_sample = Some(now);
        }
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
                let user = process
                    .user_id()
                    .and_then(|uid| self.users.get_user_by_id(uid))
                    .map(|user| user.name().to_string());
                let raw = raw_process(process, user);
                let seen = self.seen.get(&raw.pid).map_or(0, |&(_, count)| count);
                to_info(raw, seen >= REFRESHES_FOR_CPU, self.current_uid)
            })
            .collect();
        ProcessSnapshot {
            processes,
            sampled_at: unix_now(),
            cpu_measured: self.refreshes >= REFRESHES_FOR_CPU,
            system_usage: SystemUsage::from_readings(
                if self.system.cpus().is_empty() {
                    f32::NAN
                } else {
                    self.system.global_cpu_usage()
                },
                self.system_cpu_samples >= 2,
                self.system.total_memory(),
                self.system.used_memory(),
            ),
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
    fn system_usage_preserves_warmup_and_memory_availability() {
        let first = SystemUsage::from_readings(0.0, false, 1000, 250);
        assert_eq!(first.cpu_percent, None);
        assert_eq!(first.memory_used_bytes, Some(250));
        assert_eq!(first.memory_total_bytes, Some(1000));
        let unavailable = SystemUsage::from_readings(f32::NAN, true, 0, 0);
        assert_eq!(unavailable, SystemUsage::default());
        assert_eq!(
            SystemUsage::from_readings(120.0, true, 10, 11).cpu_percent,
            Some(100.0)
        );
        assert_eq!(
            SystemUsage::from_readings(1.0, true, 10, 11).memory_used_bytes,
            None
        );
    }

    #[test]
    fn system_usage_is_live_without_changing_cli_json() {
        let mut sampler = Sampler::new();
        let first = sampler.sample();
        assert_eq!(first.system_usage.cpu_percent, None);
        assert!(first
            .system_usage
            .memory_total_bytes
            .is_some_and(|total| total > 0));
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let next = sampler.sample();
        assert!(next
            .system_usage
            .cpu_percent
            .is_some_and(|cpu| (0.0..=100.0).contains(&cpu)));
        let json = serde_json::to_value(next).unwrap();
        assert!(json.get("systemUsage").is_none());
    }

    #[test]
    fn executable_path_refreshes_after_same_identity_exec() {
        use std::io::Write;
        use std::process::{Command, Stdio};
        use std::time::Duration;

        let mut child = Command::new("/bin/sh")
            .args(["-c", "read marker; exec /bin/sleep 30"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = child.id();
        let mut sampler = Sampler::new();
        let sample = sampler.sample();
        let before = sample.processes.into_iter().find(|p| p.pid == pid);
        let write_result = child.stdin.take().unwrap().write_all(b"go\n");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut after;
        loop {
            let sample = sampler.sample();
            after = sample.processes.into_iter().find(|p| p.pid == pid);
            let changed = after
                .as_ref()
                .and_then(|p| p.exe.as_ref())
                .and_then(|path| path.file_name())
                .is_some_and(|name| name == "sleep");
            if changed || Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        // Clean up only our child before making assertions, including the failing case.
        child.kill().unwrap();
        child.wait().unwrap();
        write_result.unwrap();
        let before = before.expect("shell appears before exec");
        let after = after.expect("same process appears after exec");
        assert_eq!(before.pid, after.pid);
        assert_eq!(before.start_time, after.start_time);
        assert!(before.exe.is_some());
        assert_ne!(
            before.exe.as_ref().and_then(|path| path.file_name()),
            Some(std::ffi::OsStr::new("sleep"))
        );
        assert_eq!(
            after.exe.as_ref().and_then(|path| path.file_name()),
            Some(std::ffi::OsStr::new("sleep"))
        );
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
