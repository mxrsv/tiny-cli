//! Read-only process queries exported to Swift. Mirror records keep
//! `tiny-core` types off the boundary.

use tiny_core::processes::{self, ListeningPort, ParentState, ProcessInfo, ProcessSnapshot};
use tiny_core::runner::RealRunner;

use crate::{FfiError, TinySession};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: Option<String>,
    pub is_current_user: bool,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
    /// Per-core percentage; can exceed 100. `None` when not measured.
    pub cpu_percent: Option<f32>,
    pub cpu_measured: bool,
    pub memory_bytes: Option<u64>,
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiProcessSnapshot {
    pub processes: Vec<FfiProcessInfo>,
    pub sampled_at: u64,
    pub cpu_measured: bool,
    pub system_usage: FfiSystemUsage,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiSystemUsage {
    pub cpu_percent: Option<f32>,
    pub cpu_warming_up: bool,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum FfiParentState {
    None,
    Running { pid: u32, name: String },
    Exited { pid: u32 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiListeningPort {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct FfiProcessDetail {
    pub process: FfiProcessInfo,
    pub parent: FfiParentState,
    pub children: Vec<FfiProcessInfo>,
    pub ports: Option<Vec<FfiListeningPort>>,
    pub ports_error: Option<String>,
}

#[uniffi::export]
impl TinySession {
    /// Samples all processes. CPU is measured from a process's third sample.
    pub fn process_list(&self) -> Result<FfiProcessSnapshot, FfiError> {
        Ok(self.sampler().sample().into())
    }

    /// Resamples, then returns detail, parent/children and listening ports.
    pub fn process_detail(&self, pid: u32) -> Result<FfiProcessDetail, FfiError> {
        let snapshot = self.sampler().sample();
        let detail = processes::detail(&snapshot, pid, &RealRunner)?;
        Ok(FfiProcessDetail {
            process: detail.process.into(),
            parent: detail.parent.into(),
            children: detail.children.into_iter().map(Into::into).collect(),
            ports: detail
                .ports
                .map(|ports| ports.into_iter().map(Into::into).collect()),
            ports_error: detail.ports_error,
        })
    }
}

impl From<ProcessInfo> for FfiProcessInfo {
    fn from(info: ProcessInfo) -> Self {
        Self {
            pid: info.pid,
            name: info.name,
            user: info.user,
            is_current_user: info.is_current_user,
            parent_pid: info.parent_pid,
            start_time: info.start_time,
            cpu_percent: info.cpu_percent,
            cpu_measured: info.cpu_measured,
            memory_bytes: info.memory_bytes,
            executable_path: info
                .exe
                .and_then(|path| path.into_os_string().into_string().ok()),
        }
    }
}

impl From<ProcessSnapshot> for FfiProcessSnapshot {
    fn from(snapshot: ProcessSnapshot) -> Self {
        Self {
            processes: snapshot.processes.into_iter().map(Into::into).collect(),
            sampled_at: snapshot.sampled_at,
            cpu_measured: snapshot.cpu_measured,
            system_usage: FfiSystemUsage {
                cpu_percent: snapshot.system_usage.cpu_percent,
                cpu_warming_up: snapshot.system_usage.cpu_warming_up,
                memory_used_bytes: snapshot.system_usage.memory_used_bytes,
                memory_total_bytes: snapshot.system_usage.memory_total_bytes,
            },
        }
    }
}

impl From<ParentState> for FfiParentState {
    fn from(parent: ParentState) -> Self {
        match parent {
            ParentState::None => Self::None,
            ParentState::Running { pid, name } => Self::Running { pid, name },
            ParentState::Exited { pid } => Self::Exited { pid },
        }
    }
}

impl From<ListeningPort> for FfiListeningPort {
    fn from(port: ListeningPort) -> Self {
        Self {
            address: port.address,
            port: port.port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_is_unmeasured_until_the_third_list() {
        let session = TinySession::new();
        let first = session.process_list().unwrap();
        assert!(!first.cpu_measured);
        assert!(first.processes.iter().all(|p| p.cpu_percent.is_none()));
        std::thread::sleep(sysinfo_interval());
        assert!(!session.process_list().unwrap().cpu_measured);
        std::thread::sleep(sysinfo_interval());
        let third = session.process_list().unwrap();
        assert!(third.cpu_measured);
        let me = third
            .processes
            .iter()
            .find(|p| p.pid == std::process::id())
            .expect("own process is sampled");
        assert!(me.is_current_user);
        assert!(me.cpu_measured);
    }

    #[test]
    fn detail_of_own_process_has_ports_and_unknown_pid_is_typed() {
        let session = TinySession::new();
        let detail = session.process_detail(std::process::id()).unwrap();
        assert!(detail.ports.is_some() || detail.ports_error.is_some());
        assert!(matches!(
            session.process_detail(u32::MAX),
            Err(FfiError::InvalidInput { .. })
        ));
    }

    #[test]
    fn process_queries_bypass_the_operation_gate() {
        let session = TinySession::new();
        let _gate = session.begin().unwrap();
        assert!(!session.process_list().unwrap().processes.is_empty());
    }

    fn sysinfo_interval() -> std::time::Duration {
        std::time::Duration::from_millis(250)
    }
}
