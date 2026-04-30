use anyhow::Result;

use crate::cli::CleanOpts;
use crate::util::format_bytes;

mod cli_validate;
mod discover;
mod execute;
pub mod fs_safe;
mod picker;
mod process;
pub mod providers;
mod report;
mod types;

use discover::CategoryGroup;
use types::CleanAction;

pub fn run(opts: CleanOpts) -> Result<()> {
    cli_validate::validate(&opts)?;

    let discovery = discover::discover(&opts)?;

    println!("Cleanup candidates");
    println!();
    report::print_summary(&discovery);

    if discovery.groups.is_empty() {
        return Ok(());
    }

    if opts.dry_run {
        let refs: Vec<&CategoryGroup> = discovery.groups.iter().collect();
        report::print_plan(&refs);
        println!();
        println!("(dry-run — no changes made)");
        return Ok(());
    }

    let selected: Vec<&CategoryGroup> = if opts.yes {
        // -y + --category: skip picker entirely, plan = the named categories.
        discovery.groups.iter().collect()
    } else {
        let indices = picker::pick_categories(&discovery.groups)?;
        if indices.is_empty() {
            println!("No categories selected.");
            return Ok(());
        }
        indices.into_iter().map(|i| &discovery.groups[i]).collect()
    };

    report::print_plan(&selected);

    let action = decide_action(&opts, &selected)?;
    match action {
        CleanAction::DryRun => {
            println!();
            println!("(dry-run — no changes made)");
            Ok(())
        }
        CleanAction::Cancel => {
            println!("Aborted.");
            Ok(())
        }
        CleanAction::Trash | CleanAction::HardDelete => {
            let report = execute::execute(&selected, action)?;
            print_exec_report(&report);
            Ok(())
        }
    }
}

fn decide_action(opts: &CleanOpts, selected: &[&CategoryGroup]) -> Result<CleanAction> {
    if opts.yes {
        // Validation already enforced --hard + TINY_CONFIRM_HARD=1 if --hard.
        return Ok(if opts.hard {
            CleanAction::HardDelete
        } else {
            CleanAction::Trash
        });
    }
    let prefer_hard = opts.hard;
    let action = picker::pick_action(prefer_hard)?;
    if matches!(action, CleanAction::HardDelete) {
        let total: u64 = selected.iter().map(|g| g.total_size).sum();
        let count: usize = selected.iter().map(|g| g.items.len()).sum();
        if !picker::confirm_hard_delete(count, total)? {
            return Ok(CleanAction::Cancel);
        }
    }
    Ok(action)
}

fn print_exec_report(report: &types::ExecReport) {
    println!();
    let removed = report.removed_paths.len();
    let total_size: u64 = 0; // size is not retained per-path; summary by count only
    let _ = total_size;
    println!(
        "✓ removed {} path{}",
        removed,
        if removed == 1 { "" } else { "s" }
    );
    if !report.failed.is_empty() {
        println!();
        println!("Failures ({}):", report.failed.len());
        for (path, err) in &report.failed {
            println!("  ✗ {}: {}", path.display(), err);
        }
    }
    if let Some(app) = &report.skipped_running_app {
        println!();
        println!("⚠ skipped because {} is running. Quit {} and rerun.", app, app);
    }
    let _ = format_bytes; // keep import warning quiet for now
}
