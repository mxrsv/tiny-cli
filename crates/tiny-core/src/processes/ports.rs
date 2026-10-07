//! Listening TCP ports for one process via a bounded `lsof` probe.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::runner::CommandRunner;

pub const LSOF: &str = "/usr/sbin/lsof";
pub const LSOF_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListeningPort {
    pub address: String,
    pub port: u16,
}

/// Returns the process's listening TCP ports, or a diagnostic when the probe
/// fails. `lsof` exits 1 with no output when nothing matches, which means
/// "no listening ports", not a failure.
pub fn listening_ports(pid: u32, runner: &dyn CommandRunner) -> Result<Vec<ListeningPort>, String> {
    let pid = pid.to_string();
    let args = ["-nP", "-a", "-p", &pid, "-iTCP", "-sTCP:LISTEN", "-Fn"];
    let outcome = runner
        .output(LSOF, &args, LSOF_TIMEOUT)
        .map_err(|error| error.to_string())?;
    let nothing_matched = outcome.status == Some(1)
        && outcome.stdout.trim().is_empty()
        && outcome.stderr.trim().is_empty();
    if nothing_matched {
        return Ok(Vec::new());
    }
    if !outcome.success() {
        return Err(format!(
            "lsof exited with status {:?}: {}",
            outcome.status,
            outcome.stderr.trim()
        ));
    }
    Ok(parse_lsof_names(&outcome.stdout))
}

/// Parses `lsof -Fn` output: only `n` lines carry `address:port`. Splits on
/// the last `:` so `[::1]:3000` and `*:8080` both parse.
fn parse_lsof_names(stdout: &str) -> Vec<ListeningPort> {
    let mut ports: Vec<ListeningPort> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix('n'))
        .filter_map(|name| {
            let (address, port) = name.rsplit_once(':')?;
            Some(ListeningPort {
                address: address.to_string(),
                port: port.parse().ok()?,
            })
        })
        .collect();
    ports.sort();
    ports.dedup();
    ports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::test_support::MockRunner;
    use crate::runner::{CommandError, CommandOutcome};

    const ARGS: [&str; 7] = ["-nP", "-a", "-p", "42", "-iTCP", "-sTCP:LISTEN", "-Fn"];

    fn outcome(status: i32, stdout: &str, stderr: &str) -> CommandOutcome {
        CommandOutcome {
            status: Some(status),
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    #[test]
    fn parses_ipv4_ipv6_and_wildcard_listeners() {
        let stdout = "p42\nf5\nn*:8080\nf6\nn127.0.0.1:5173\nf7\nn[::1]:3000\nf8\nn*:8080\n";
        let runner = MockRunner::new().with_output(LSOF, &ARGS, Ok(outcome(0, stdout, "")));
        assert_eq!(
            listening_ports(42, &runner).unwrap(),
            vec![
                ListeningPort {
                    address: "*".into(),
                    port: 8080
                },
                ListeningPort {
                    address: "127.0.0.1".into(),
                    port: 5173
                },
                ListeningPort {
                    address: "[::1]".into(),
                    port: 3000
                },
            ]
        );
    }

    #[test]
    fn exit_one_without_output_means_no_ports() {
        let runner = MockRunner::new().with_output(LSOF, &ARGS, Ok(outcome(1, "", "")));
        assert_eq!(listening_ports(42, &runner).unwrap(), Vec::new());
    }

    #[test]
    fn exit_one_with_stderr_is_an_error() {
        let runner = MockRunner::new().with_output(
            LSOF,
            &ARGS,
            Ok(outcome(1, "", "lsof: permission denied")),
        );
        assert!(listening_ports(42, &runner)
            .unwrap_err()
            .contains("permission denied"));
    }

    #[test]
    fn timeout_is_an_error_not_an_empty_list() {
        let timeout = CommandError::Timeout {
            bin: LSOF.into(),
            timeout_ms: 3000,
        };
        let runner = MockRunner::new().with_output(LSOF, &ARGS, Err(timeout));
        assert!(listening_ports(42, &runner)
            .unwrap_err()
            .contains("did not finish"));
    }
}
