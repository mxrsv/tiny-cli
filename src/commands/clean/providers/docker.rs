use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};

use super::CleanProvider;
use crate::commands::clean::runner::{CommandRunner, RealRunner};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "docker";
const LABEL: &str = "Docker images/build/volumes";
const APP: &str = "Docker Desktop";

/// Synthetic placeholders so `CleanItem.path` stays meaningful in the UI
/// even though there's no filesystem entry to remove. `execute()` reads
/// these back to decide what to prune.
const PLACEHOLDER_IMAGES: &str = "<docker:images>";
const PLACEHOLDER_BUILD: &str = "<docker:build-cache>";
const PLACEHOLDER_VOLUMES: &str = "<docker:volumes>";

pub struct Docker {
    runner: Arc<dyn CommandRunner>,
}

impl Docker {
    pub fn new() -> Self {
        Self {
            runner: Arc::new(RealRunner),
        }
    }
    #[cfg(test)]
    pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
        Self { runner }
    }
}

impl Default for Docker {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanProvider for Docker {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn requires_app_quit(&self) -> Option<&'static str> {
        Some(APP)
    }
    fn available(&self) -> bool {
        self.runner.which("docker")
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        // `docker system df --format '{{json .}}'` emits one JSON object
        // per resource type (Images, Containers, Local Volumes, Build
        // Cache). Daemon-down → non-zero exit → graceful empty.
        let out = self.runner.run("docker", &["system", "df", "--format", "{{json .}}"]);
        if !out.success {
            eprintln!("warn: docker daemon unavailable, skipping");
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        for line in out.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some((kind, size)) = parse_df_line(trimmed) {
                let placeholder = match kind {
                    DfKind::Images => PLACEHOLDER_IMAGES,
                    DfKind::BuildCache => PLACEHOLDER_BUILD,
                    DfKind::Volumes => PLACEHOLDER_VOLUMES,
                };
                if size == 0 {
                    continue;
                }
                items.push(CleanItem {
                    category_id: ID.to_string(),
                    category_label: LABEL.to_string(),
                    path: PathBuf::from(placeholder),
                    size,
                    risk: RiskLevel::Review,
                });
            }
        }
        Ok(items)
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        if matches!(action, ExecAction::EmptyTrash) {
            return Err(anyhow!("docker provider does not accept EmptyTrash"));
        }
        if items.is_empty() {
            return Ok(ExecReport::default());
        }
        // Docker has no Trash semantics. Both Trash and HardDelete map to
        // the same prune command; warn user when they picked Trash.
        if matches!(action, ExecAction::Trash) {
            eprintln!(
                "warn: docker prune is not recoverable (Docker has no Trash); proceeding"
            );
        }
        let out = self
            .runner
            .run("docker", &["system", "prune", "-af", "--volumes"]);
        let mut report = ExecReport::default();
        if out.success {
            for item in items {
                report.removed_paths.push(item.path.clone());
            }
        } else {
            for item in items {
                report
                    .failed
                    .push((item.path.clone(), "docker system prune failed".into()));
            }
        }
        Ok(report)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DfKind {
    Images,
    BuildCache,
    Volumes,
}

/// Parses one line of `docker system df --format '{{json .}}'`. The JSON
/// shape is `{"Type":"Images","Size":"1.234GB",...}`; we only need Type +
/// Size. Returns None for non-cleanable types (Containers).
///
/// Hand-rolled to avoid adding a serde_json dep just for this.
fn parse_df_line(line: &str) -> Option<(DfKind, u64)> {
    let kind = if line.contains("\"Type\":\"Images\"") {
        DfKind::Images
    } else if line.contains("\"Type\":\"Build Cache\"") {
        DfKind::BuildCache
    } else if line.contains("\"Type\":\"Local Volumes\"") {
        DfKind::Volumes
    } else {
        return None;
    };
    let size_str = extract_quoted(line, "Size")?;
    let size = parse_human_size(size_str)?;
    Some((kind, size))
}

/// Extracts the value of `"<key>":"<value>"` from `s`. Naive, only used
/// for trusted docker JSON output.
fn extract_quoted<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\":\"", key);
    let start = s.find(&needle)? + needle.len();
    let rest = &s[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// Parses `1.23GB` / `456MB` / `0B` / `100kB` into bytes. docker uses
/// powers of 1000 for "GB"/"MB"/etc per its CLI convention.
fn parse_human_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num_part, unit_part): (String, String) = s
        .chars()
        .partition(|c| c.is_ascii_digit() || *c == '.');
    let num: f64 = num_part.parse().ok()?;
    let mult: f64 = match unit_part.trim() {
        "B" | "" => 1.0,
        "kB" | "KB" => 1_000.0,
        "MB" => 1_000_000.0,
        "GB" => 1_000_000_000.0,
        "TB" => 1_000_000_000_000.0,
        _ => return None,
    };
    Some((num * mult) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::providers::known_category_ids;
    use crate::commands::clean::runner::test_support::MockRunner;

    #[test]
    fn docker_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn docker_gates_desktop() {
        let p = Docker::new();
        assert_eq!(p.requires_app_quit(), Some(APP));
    }

    #[test]
    fn parse_df_line_extracts_images_and_size() {
        let line = r#"{"Type":"Images","TotalCount":"5","Active":"3","Size":"2.5GB","Reclaimable":"1.2GB"}"#;
        let (kind, size) = parse_df_line(line).unwrap();
        assert_eq!(kind, DfKind::Images);
        assert_eq!(size, 2_500_000_000);
    }

    #[test]
    fn parse_df_line_returns_none_for_containers() {
        let line = r#"{"Type":"Containers","Size":"0B"}"#;
        assert!(parse_df_line(line).is_none());
    }

    #[test]
    fn parse_human_size_handles_units() {
        assert_eq!(parse_human_size("0B"), Some(0));
        assert_eq!(parse_human_size("100B"), Some(100));
        assert_eq!(parse_human_size("1kB"), Some(1_000));
        assert_eq!(parse_human_size("2MB"), Some(2_000_000));
        assert_eq!(parse_human_size("1.5GB"), Some(1_500_000_000));
        assert_eq!(parse_human_size("garbage"), None);
    }

    #[test]
    fn docker_daemon_down_returns_empty_safely() {
        // which("docker") succeeds (CLI installed) but `system df` fails
        // (daemon not running). Provider must NOT propagate error.
        let runner = Arc::new(
            MockRunner::new()
                .with_which("docker")
                .with_response("docker", &["system", "df", "--format", "{{json .}}"], false, ""),
        );
        let p = Docker::with_runner(runner);
        let items = p.discover().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn docker_unavailable_when_cli_missing() {
        let runner = Arc::new(MockRunner::new());
        let p = Docker::with_runner(runner);
        assert!(!p.available());
    }

    #[test]
    fn docker_discover_parses_multi_line_output() {
        let out = "\
{\"Type\":\"Images\",\"Size\":\"1GB\"}
{\"Type\":\"Containers\",\"Size\":\"0B\"}
{\"Type\":\"Local Volumes\",\"Size\":\"500MB\"}
{\"Type\":\"Build Cache\",\"Size\":\"2GB\"}
";
        let runner = Arc::new(
            MockRunner::new()
                .with_which("docker")
                .with_response(
                    "docker",
                    &["system", "df", "--format", "{{json .}}"],
                    true,
                    out,
                ),
        );
        let p = Docker::with_runner(runner);
        let items = p.discover().unwrap();
        // 3 items (Images, Local Volumes, Build Cache); Containers skipped.
        assert_eq!(items.len(), 3);
        let total: u64 = items.iter().map(|i| i.size).sum();
        assert_eq!(total, 1_000_000_000 + 500_000_000 + 2_000_000_000);
    }
}
