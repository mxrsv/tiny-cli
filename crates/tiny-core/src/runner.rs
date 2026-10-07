//! Subprocess abstraction for code that shells out (`docker`, `go`, `lsof`,
//! `tmutil`, `mdfind`, `defaults`, `getconf`, ...). Real code uses
//! `RealRunner`; tests inject `MockRunner` so they don't depend on which
//! CLIs are installed in CI.
//!
//! `which`/`run` keep the original collapsing semantics that providers rely on.
//! New code uses `output`, which is bounded by a timeout and reports failures
//! as typed errors instead of empty results.

use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
}

/// A command that ran to completion. A non-zero exit is not an error here:
/// some tools (for example `lsof`) exit 1 to mean "nothing matched".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    /// Exit code; `None` when the process was terminated by a signal.
    pub status: Option<i32>,
    pub stdout: String,
    /// Lossy UTF-8, truncated to `STDERR_LIMIT` bytes.
    pub stderr: String,
}

impl CommandOutcome {
    pub fn success(&self) -> bool {
        self.status == Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    #[error("{0} was not found")]
    NotFound(String),
    #[error("failed to run {bin}: {detail}")]
    Spawn { bin: String, detail: String },
    #[error("{bin} did not finish within {timeout_ms} ms")]
    Timeout { bin: String, timeout_ms: u64 },
    #[error("{0} produced non-UTF-8 output")]
    NonUtf8(String),
}

/// Maximum stderr kept per command; the rest is drained and discarded.
pub const STDERR_LIMIT: u64 = 4096;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub trait CommandRunner: Send + Sync {
    /// Returns true iff `which <bin>` succeeds.
    fn which(&self, bin: &str) -> bool;

    /// Runs `bin args...`, returning `(success, stdout)`. Failures (spawn
    /// error, non-utf8 output, non-zero exit) collapse to `success=false`,
    /// `stdout=""`.
    fn run(&self, bin: &str, args: &[&str]) -> CommandOutput;

    /// Runs `bin args...` with stdin closed, killing it after `timeout`.
    fn output(
        &self,
        bin: &str,
        args: &[&str],
        timeout: Duration,
    ) -> Result<CommandOutcome, CommandError>;
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

    fn output(
        &self,
        bin: &str,
        args: &[&str],
        timeout: Duration,
    ) -> Result<CommandOutcome, CommandError> {
        let spawn_error = |error: io::Error| match error.kind() {
            io::ErrorKind::NotFound => CommandError::NotFound(bin.to_string()),
            _ => CommandError::Spawn {
                bin: bin.to_string(),
                detail: error.to_string(),
            },
        };
        let mut child = Command::new(bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(spawn_error)?;
        // Drain both pipes before waiting so a chatty child cannot block on a
        // full pipe buffer.
        let stdout = drain(child.stdout.take(), u64::MAX);
        let stderr = drain(child.stderr.take(), STDERR_LIMIT);
        let deadline = Instant::now() + timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    // ponytail: drain threads are detached here; a grandchild
                    // that inherited the pipes keeps them alive until it exits.
                    return Err(CommandError::Timeout {
                        bin: bin.to_string(),
                        timeout_ms: timeout.as_millis() as u64,
                    });
                }
                Ok(None) => thread::sleep(POLL_INTERVAL),
                Err(error) => return Err(spawn_error(error)),
            }
        };
        let stdout =
            String::from_utf8(join(stdout)).map_err(|_| CommandError::NonUtf8(bin.to_string()))?;
        let stderr = String::from_utf8_lossy(&join(stderr)).into_owned();
        Ok(CommandOutcome {
            status: status.code(),
            stdout,
            stderr,
        })
    }
}

/// Reads up to `limit` bytes on a background thread and discards the rest so
/// the child never blocks on a full pipe.
fn drain<R: Read + Send + 'static>(pipe: Option<R>, limit: u64) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut kept = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = (&mut pipe).take(limit).read_to_end(&mut kept);
            let _ = io::copy(&mut pipe, &mut io::sink());
        }
        kept
    })
}

fn join(handle: JoinHandle<Vec<u8>>) -> Vec<u8> {
    handle.join().unwrap_or_default()
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Maps `(bin, args.join(" "))` to canned results. Unknown `run`
    /// invocations return `(false, "")`; unknown `output` invocations return
    /// `NotFound`. `output_calls` records every `output` invocation.
    pub struct MockRunner {
        pub which_set: Mutex<Vec<String>>,
        pub responses: Mutex<HashMap<String, (bool, String)>>,
        pub outputs: Mutex<HashMap<String, Result<CommandOutcome, CommandError>>>,
        pub output_calls: Mutex<Vec<String>>,
    }

    impl MockRunner {
        pub fn new() -> Self {
            Self {
                which_set: Mutex::new(Vec::new()),
                responses: Mutex::new(HashMap::new()),
                outputs: Mutex::new(HashMap::new()),
                output_calls: Mutex::new(Vec::new()),
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
        pub fn with_output(
            self,
            bin: &str,
            args: &[&str],
            result: Result<CommandOutcome, CommandError>,
        ) -> Self {
            let key = format!("{} {}", bin, args.join(" "));
            self.outputs.lock().unwrap().insert(key, result);
            self
        }
    }

    impl Default for MockRunner {
        fn default() -> Self {
            Self::new()
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
        fn output(
            &self,
            bin: &str,
            args: &[&str],
            _timeout: Duration,
        ) -> Result<CommandOutcome, CommandError> {
            let key = format!("{} {}", bin, args.join(" "));
            self.output_calls.lock().unwrap().push(key.clone());
            self.outputs
                .lock()
                .unwrap()
                .get(&key)
                .cloned()
                .unwrap_or_else(|| Err(CommandError::NotFound(bin.to_string())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SH: &str = "/bin/sh";
    const GENEROUS: Duration = Duration::from_secs(5);

    #[test]
    fn output_reports_exit_code_stdout_and_stderr() {
        let outcome = RealRunner
            .output(SH, &["-c", "echo out; echo err >&2; exit 3"], GENEROUS)
            .expect("sh runs");
        assert_eq!(outcome.status, Some(3));
        assert!(!outcome.success());
        assert_eq!(outcome.stdout, "out\n");
        assert_eq!(outcome.stderr, "err\n");
    }

    #[test]
    fn output_times_out_and_kills_the_child() {
        let started = Instant::now();
        let result = RealRunner.output(SH, &["-c", "sleep 5"], Duration::from_millis(100));
        assert!(matches!(
            result,
            Err(CommandError::Timeout {
                timeout_ms: 100,
                ..
            })
        ));
        assert!(started.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn output_reports_missing_binary() {
        let result = RealRunner.output("/nonexistent/tiny-test-binary", &[], GENEROUS);
        assert!(matches!(result, Err(CommandError::NotFound(_))));
    }

    #[test]
    fn output_rejects_non_utf8_stdout() {
        let result = RealRunner.output(SH, &["-c", "printf '\\377'"], GENEROUS);
        assert!(matches!(result, Err(CommandError::NonUtf8(_))));
    }

    #[test]
    fn output_caps_stderr_without_blocking() {
        let outcome = RealRunner
            .output(
                SH,
                &["-c", "head -c 200000 /dev/zero | tr '\\0' x >&2"],
                GENEROUS,
            )
            .expect("sh runs");
        assert!(outcome.success());
        assert_eq!(outcome.stderr.len() as u64, STDERR_LIMIT);
    }
}
