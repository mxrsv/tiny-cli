use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};

use crate::util::format_bytes;

use super::discover::CategoryGroup;
use super::report::truncate;
use super::types::{CleanAction, RiskLevel};

/// Show the multi-select picker. Safe categories are pre-checked; Review and
/// Destructive are listed but unchecked. Returns indices into `groups`.
pub fn pick_categories(groups: &[CategoryGroup]) -> Result<Vec<usize>> {
    if groups.is_empty() {
        return Ok(Vec::new());
    }
    let labels: Vec<String> = groups
        .iter()
        .map(|g| {
            format!(
                "{:<28} {:>10}   {}",
                truncate(&g.label, 28),
                format_bytes(g.total_size),
                g.risk.badge()
            )
        })
        .collect();
    let defaults: Vec<bool> = groups
        .iter()
        .map(|g| matches!(g.risk, RiskLevel::Safe))
        .collect();
    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select categories to clean (Space to toggle, Enter to confirm)")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .context("category picker failed")?;
    Ok(selected)
}

/// Action menu shown after the plan is printed. Mirrors `tiny uninstall`.
pub fn pick_action(prefer_hard: bool) -> Result<CleanAction> {
    let items = [
        "Move to Trash (recoverable)",
        "Dry-run (no changes)",
        "Hard delete (NOT recoverable)",
        "Cancel",
    ];
    let default_idx = if prefer_hard { 2 } else { 0 };
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What to do?")
        .items(&items)
        .default(default_idx)
        .interact()
        .context("action menu failed")?;
    Ok(match idx {
        0 => CleanAction::Trash,
        1 => CleanAction::DryRun,
        2 => CleanAction::HardDelete,
        _ => CleanAction::Cancel,
    })
}

pub fn confirm_hard_delete(item_count: usize, total_size: u64) -> Result<bool> {
    let prompt = format!(
        "PERMANENTLY DELETE {} item{} ({}) — this CANNOT be undone. Are you sure?",
        item_count,
        if item_count == 1 { "" } else { "s" },
        format_bytes(total_size),
    );
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(false)
        .interact()
        .context("hard-delete confirm prompt failed")
}
