//! Subprocess abstraction for providers that shell out (`docker`, `go`,
//! `tmutil`, `mdfind`, `defaults`, `getconf`, ...). Real provider code uses
//! `RealRunner`; tests inject `MockRunner` so they don't depend on which
//! CLIs are installed in CI.
//!
//! All methods take `&[&str]` args and return owned strings. Stderr is
//! discarded — providers that need diagnostics should log via `eprintln!`
//! at their own level.

use std::process::Command;

pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
}

pub trait CommandRunner: Send + Sync {
    /// Returns true iff `which <bin>` succeeds.
    fn which(&self, bin: &str) -> bool;

    /// Runs `bin args...`, returning `(success, stdout)`. Failures (spawn
    /// error, non-utf8 output, non-zero exit) collapse to `success=false`,
    /// `stdout=""`.
    fn run(&self, bin: &str, args: &[&str]) -> CommandOutput;
}

pub struct RealRunner;

impl CommandRunner for RealRunner {
    fn which(&self, bin: &str) -> bool {
        Command::new("which")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn run(&self, bin: &str, args: &[&str]) -> CommandOutput {
        let out = match Command::new(bin).args(args).output() {
            Ok(o) => o,
            Err(_) => {
                return CommandOutput {
                    success: false,
                    stdout: String::new(),
                }
            }
        };
        let stdout = String::from_utf8(out.stdout).unwrap_or_default();
        CommandOutput {
            success: out.status.success(),
            stdout,
        }
    }
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Maps `(bin, args.join(" "))` to `(success, stdout)`. Unknown
    /// invocations return `(false, "")`.
    pub struct MockRunner {
        pub which_set: Mutex<Vec<String>>,
        pub responses: Mutex<HashMap<String, (bool, String)>>,
    }

    impl MockRunner {
        pub fn new() -> Self {
            Self {
                which_set: Mutex::new(Vec::new()),
                responses: Mutex::new(HashMap::new()),
            }
        }
        pub fn with_which(self, bin: &str) -> Self {
            self.which_set.lock().unwrap().push(bin.to_string());
            self
        }
        pub fn with_response(self, bin: &str, args: &[&str], success: bool, stdout: &str) -> Self {
            let key = format!("{} {}", bin, args.join(" "));
            self.responses
                .lock()
                .unwrap()
                .insert(key, (success, stdout.to_string()));
            self
        }
    }

    impl CommandRunner for MockRunner {
        fn which(&self, bin: &str) -> bool {
            self.which_set.lock().unwrap().iter().any(|b| b == bin)
        }
        fn run(&self, bin: &str, args: &[&str]) -> CommandOutput {
            let key = format!("{} {}", bin, args.join(" "));
            match self.responses.lock().unwrap().get(&key) {
                Some((success, stdout)) => CommandOutput {
                    success: *success,
                    stdout: stdout.clone(),
                },
                None => CommandOutput {
                    success: false,
                    stdout: String::new(),
                },
            }
        }
    }
}
