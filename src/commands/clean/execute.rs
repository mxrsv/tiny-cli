//! Execute a plan against per-provider executors.
//!
//! Translates the user-facing `CleanAction` into provider-specific
//! `ExecAction`s:
//! - `CleanAction::Trash` → `ExecAction::Trash` for every provider.
//!   Trash provider rejects this; we therefore refuse `Trash` whenever a
//!   `trash` group is in the plan (the user must pick Hard delete instead).
//! - `CleanAction::HardDelete` → `ExecAction::EmptyTrash` for the Trash
//!   provider, `ExecAction::HardDelete` for everyone else.

use anyhow::{anyhow, Result};

use super::discover::CategoryGroup;
use super::providers::{all_providers, CleanProvider};
use super::types::{CleanAction, ExecAction, ExecReport};

const TRASH_ID: &str = "trash";

pub fn execute(groups: &[&CategoryGroup], action: CleanAction) -> Result<ExecReport> {
    let has_trash = groups.iter().any(|g| g.id == TRASH_ID);
    if has_trash && matches!(action, CleanAction::Trash) {
        return Err(anyhow!(
            "trash category requires --hard (cannot 'move trash to trash')"
        ));
    }

    let providers = all_providers();
    let mut report = ExecReport::default();

    for group in groups {
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
        let group_report = provider.execute(&group.items, exec_action)?;
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
