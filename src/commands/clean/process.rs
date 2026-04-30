//! Running-app detection.
//!
//! Wraps `pgrep -x <name>` (exit 0 = at least one matching process) behind a
//! `ProcessChecker` trait so tests can inject deterministic answers.

use std::process::Command;

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
    use super::ProcessChecker;
    use std::collections::HashSet;

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
    fn any_running_combines_names() {
        let m = MockChecker::with_running(["rustc"]);
        assert!(any_running(&m, &["cargo", "rustc"]));
        assert!(!any_running(&m, &["npm", "yarn"]));
    }
}
