//! Running-app detection.
//!
//! Wraps `pgrep -x <name>` (exit 0 = at least one matching process) behind a
//! `ProcessChecker` trait so tests can inject deterministic answers.
//! `ProcessChecker` keeps the CLI's lenient semantics (a failed `pgrep` reads
//! as "not running"); checked code uses the fallible `AppProbe`.

use std::process::Command;
use std::time::Duration;

use crate::runner::CommandRunner;

/// Absolute so a Finder-launched app does not depend on its minimal `PATH`.
const PGREP: &str = "/usr/bin/pgrep";
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Fallible running-app check. `Err` means the answer is unknown, and a
/// caller acting on the result must refuse rather than assume "not running".
pub trait AppProbe {
    fn probe(&self, name: &str) -> Result<bool, String>;
}

/// `pgrep -x` through the bounded runner: exit 0 = running, exit 1 = not
/// running, anything else (other exit, spawn failure, timeout) = unknown.
pub struct RunnerProbe<'r>(pub &'r dyn CommandRunner);

impl AppProbe for RunnerProbe<'_> {
    fn probe(&self, name: &str) -> Result<bool, String> {
        let outcome = self
            .0
            .output(PGREP, &["-x", name], PROBE_TIMEOUT)
            .map_err(|e| e.to_string())?;
        match outcome.status {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            status => Err(format!(
                "pgrep exited with {status:?}: {}",
                outcome.stderr.trim()
            )),
        }
    }
}

impl AppProbe for PgrepChecker {
    fn probe(&self, name: &str) -> Result<bool, String> {
        Ok(self.is_running(name))
    }
}

pub trait ProcessChecker {
    /// Returns true if any process is currently running with the given name.
    fn is_running(&self, name: &str) -> bool;
}

pub struct PgrepChecker;

impl ProcessChecker for PgrepChecker {
    fn is_running(&self, name: &str) -> bool {
        Command::new("pgrep")
            .args(["-x", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Runtime helper used by providers when they only need a one-shot check.
/// Tests should construct their own `ProcessChecker` instead of calling this.
pub fn is_running(name: &str) -> bool {
    PgrepChecker.is_running(name)
}

/// Returns true if any of the given names is running. Used by providers
/// that gate on multiple process names (e.g. cargo + rustc).
#[allow(dead_code)]
pub fn any_running(checker: &dyn ProcessChecker, names: &[&str]) -> bool {
    names.iter().any(|n| checker.is_running(n))
}

#[cfg(test)]
pub mod test_support {
    use super::{AppProbe, ProcessChecker};
    use std::collections::HashSet;

    /// A probe whose answer is always unknown.
    pub struct FailingProbe;

    impl AppProbe for FailingProbe {
        fn probe(&self, _name: &str) -> Result<bool, String> {
            Err("probe unavailable".into())
        }
    }

    pub struct MockChecker {
        running: HashSet<String>,
    }

    impl MockChecker {
        pub fn with_running<I, S>(names: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Into<String>,
        {
            Self {
                running: names.into_iter().map(Into::into).collect(),
            }
        }

        pub fn none() -> Self {
            Self {
                running: HashSet::new(),
            }
        }
    }

    impl ProcessChecker for MockChecker {
        fn is_running(&self, name: &str) -> bool {
            self.running.contains(name)
        }
    }

    impl AppProbe for MockChecker {
        fn probe(&self, name: &str) -> Result<bool, String> {
            Ok(self.is_running(name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::MockChecker;
    use super::*;

    #[test]
    fn mock_reports_only_listed_names() {
        let m = MockChecker::with_running(["Xcode"]);
        assert!(m.is_running("Xcode"));
        assert!(!m.is_running("cargo"));
    }

    #[test]
    fn runner_probe_distinguishes_running_absent_and_unknown() {
        use crate::runner::test_support::MockRunner;
        use crate::runner::{CommandError, CommandOutcome};
        let exit = |code| {
            Ok(CommandOutcome {
                status: Some(code),
                stdout: String::new(),
                stderr: String::new(),
            })
        };
        let runner = MockRunner::new()
            .with_output(PGREP, &["-x", "Xcode"], exit(0))
            .with_output(PGREP, &["-x", "Mail"], exit(1))
            .with_output(PGREP, &["-x", "Bad"], exit(2))
            .with_output(
                PGREP,
                &["-x", "Slow"],
                Err(CommandError::Timeout {
                    bin: PGREP.into(),
                    timeout_ms: 5000,
                }),
            );
        let probe = RunnerProbe(&runner);
        assert_eq!(probe.probe("Xcode"), Ok(true));
        assert_eq!(probe.probe("Mail"), Ok(false));
        assert!(probe.probe("Bad").is_err());
        assert!(probe.probe("Slow").is_err());
    }

    #[test]
    fn any_running_combines_names() {
        let m = MockChecker::with_running(["rustc"]);
        assert!(any_running(&m, &["cargo", "rustc"]));
        assert!(!any_running(&m, &["npm", "yarn"]));
    }
}
