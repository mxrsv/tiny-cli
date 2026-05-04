//! Execute a plan against per-provider executors.
//!
//! Translates the user-facing `CleanAction` into provider-specific
//! `ExecAction`s:
//! - `CleanAction::Trash` → `ExecAction::Trash` for every provider.
//!   Trash provider rejects this; we therefore refuse `Trash` whenever a
//!   `trash` group is in the plan (the user must pick Hard delete instead).
//! - `CleanAction::HardDelete` → `ExecAction::EmptyTrash` for the Trash
//!   provider, `ExecAction::HardDelete` for everyone else.

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::cli::CleanOpts;

use super::discover::CategoryGroup;
use super::providers::{all_providers, CleanProvider};
use super::types::{CleanAction, CleanItem, ExecAction, ExecReport};

const TRASH_ID: &str = "trash";

pub fn execute(
    groups: &[&CategoryGroup],
    action: CleanAction,
    excluded_paths: &HashSet<PathBuf>,
    opts: &CleanOpts,
) -> Result<ExecReport> {
    let has_trash = groups.iter().any(|g| g.id == TRASH_ID);
    if has_trash && matches!(action, CleanAction::Trash) {
        return Err(anyhow!(
            "trash category requires --hard (cannot 'move trash to trash')"
        ));
    }

    let providers = all_providers(opts);
    execute_with_providers(groups, action, excluded_paths, &providers)
}

/// Same as `execute` but takes the provider list as a parameter so tests
/// can inject mocks. The public entry resolves providers via
/// `all_providers(opts)` and delegates here.
pub(crate) fn execute_with_providers(
    groups: &[&CategoryGroup],
    action: CleanAction,
    excluded_paths: &HashSet<PathBuf>,
    providers: &[Box<dyn CleanProvider>],
) -> Result<ExecReport> {
    let mut report = ExecReport::default();
    for group in groups {
        let filtered: Vec<CleanItem> = group
            .items
            .iter()
            .filter(|i| !excluded_paths.contains(&i.path))
            .cloned()
            .collect();
        if filtered.is_empty() {
            continue;
        }
        let provider = match providers.iter().find(|p| p.id() == group.id) {
            Some(p) => p,
            None => {
                return Err(anyhow!(
                    "internal error: no provider registered for category '{}'",
                    group.id
                ));
            }
        };
        let exec_action = map_action(action, group.id.as_str())?;
        let group_report = provider.execute(&filtered, exec_action)?;
        report.merge(group_report);
    }
    Ok(report)
}

/// Map a UI action to per-provider `ExecAction`. Visible to tests.
pub fn map_action(action: CleanAction, provider_id: &str) -> Result<ExecAction> {
    match action {
        CleanAction::Trash => {
            if provider_id == TRASH_ID {
                Err(anyhow!("trash provider rejects Trash action"))
            } else {
                Ok(ExecAction::Trash)
            }
        }
        CleanAction::HardDelete => {
            if provider_id == TRASH_ID {
                Ok(ExecAction::EmptyTrash)
            } else {
                Ok(ExecAction::HardDelete)
            }
        }
        CleanAction::DryRun | CleanAction::Cancel => {
            Err(anyhow!("DryRun/Cancel must be handled before execute()"))
        }
    }
}

#[allow(dead_code)]
fn ensure_provider_exists(providers: &[Box<dyn CleanProvider>], id: &str) -> Result<()> {
    if providers.iter().any(|p| p.id() == id) {
        Ok(())
    } else {
        Err(anyhow!("no provider for '{}'", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::types::RiskLevel;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct CountingProvider {
        id: &'static str,
        calls: Arc<AtomicUsize>,
        last_len: Arc<AtomicUsize>,
    }
    impl CleanProvider for CountingProvider {
        fn id(&self) -> &'static str {
            self.id
        }
        fn label(&self) -> &'static str {
            "test"
        }
        fn risk(&self) -> RiskLevel {
            RiskLevel::Safe
        }
        fn discover(&self) -> Result<Vec<CleanItem>> {
            Ok(Vec::new())
        }
        fn execute(&self, items: &[CleanItem], _action: ExecAction) -> Result<ExecReport> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.last_len.store(items.len(), Ordering::SeqCst);
            Ok(ExecReport::default())
        }
    }

    fn item(path: &str, cat: &str) -> CleanItem {
        CleanItem {
            category_id: cat.into(),
            category_label: cat.into(),
            path: PathBuf::from(path),
            size: 0,
            risk: RiskLevel::Safe,
        }
    }

    fn group_with(items: Vec<CleanItem>, id: &str) -> CategoryGroup {
        CategoryGroup {
            id: id.into(),
            label: id.into(),
            risk: RiskLevel::Safe,
            total_size: 0,
            items,
        }
    }

    #[test]
    fn drill_down_filter_removes_empty_categories() {
        let calls = Arc::new(AtomicUsize::new(0));
        let last_len = Arc::new(AtomicUsize::new(0));
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(CountingProvider {
            id: "user-logs",
            calls: calls.clone(),
            last_len: last_len.clone(),
        })];
        let g = group_with(
            vec![item("/a", "user-logs"), item("/b", "user-logs")],
            "user-logs",
        );
        let groups = vec![&g];
        let mut excluded = HashSet::new();
        excluded.insert(PathBuf::from("/a"));
        excluded.insert(PathBuf::from("/b"));
        let report =
            execute_with_providers(&groups, CleanAction::Trash, &excluded, &providers).unwrap();
        assert_eq!(report.removed_paths.len(), 0);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "provider must NOT be called when filtered set is empty"
        );
    }

    #[test]
    fn execute_filters_only_excluded_items() {
        let calls = Arc::new(AtomicUsize::new(0));
        let last_len = Arc::new(AtomicUsize::new(0));
        let providers: Vec<Box<dyn CleanProvider>> = vec![Box::new(CountingProvider {
            id: "user-logs",
            calls: calls.clone(),
            last_len: last_len.clone(),
        })];
        let g = group_with(
            vec![
                item("/a", "user-logs"),
                item("/b", "user-logs"),
                item("/c", "user-logs"),
            ],
            "user-logs",
        );
        let groups = vec![&g];
        let mut excluded = HashSet::new();
        excluded.insert(PathBuf::from("/b"));
        let _ =
            execute_with_providers(&groups, CleanAction::Trash, &excluded, &providers).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            last_len.load(Ordering::SeqCst),
            2,
            "filtered should pass 2 items, not 3"
        );
    }


    #[test]
    fn hard_delete_maps_to_empty_trash_for_trash_provider() {
        assert_eq!(
            map_action(CleanAction::HardDelete, "trash").unwrap(),
            ExecAction::EmptyTrash
        );
    }

    #[test]
    fn hard_delete_maps_to_hard_delete_for_others() {
        assert_eq!(
            map_action(CleanAction::HardDelete, "user-logs").unwrap(),
            ExecAction::HardDelete
        );
        assert_eq!(
            map_action(CleanAction::HardDelete, "cargo").unwrap(),
            ExecAction::HardDelete
        );
    }

    #[test]
    fn trash_action_rejected_for_trash_provider() {
        assert!(map_action(CleanAction::Trash, "trash").is_err());
    }

    #[test]
    fn trash_action_passes_for_others() {
        assert_eq!(
            map_action(CleanAction::Trash, "user-logs").unwrap(),
            ExecAction::Trash
        );
    }
}
