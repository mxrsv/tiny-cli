use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};

use crate::cli::{SortBy, UninstallOpts};
use crate::util::format_bytes;

const APPLICATIONS_DIR: &str = "/Applications";
const SYSTEM_BUNDLE_PREFIX: &str = "com.apple.";

pub fn run(opts: UninstallOpts) -> Result<()> {
    let targets = match opts.name.clone() {
        Some(name) => vec![resolve_app_by_name(&name)?],
        None => pick_interactive(&opts)?,
    };

    if targets.is_empty() {
        println!("No apps selected.");
        return Ok(());
    }

    let plans: Vec<Plan> = targets
        .iter()
        .map(|app| build_plan(app, &opts))
        .collect::<Result<_>>()?;

    print_plans(&plans, &opts);

    let blocked: Vec<&Plan> = plans.iter().filter(|p| p.blocked.is_some()).collect();
    if !blocked.is_empty() {
        anyhow::bail!(
            "{} app(s) blocked from removal — see report above. Use --force to override Homebrew warning.",
            blocked.len()
        );
    }

    let action = decide_action(&opts)?;
    match action {
        Action::DryRun => {
            println!();
            println!("(dry-run — no changes made)");
            Ok(())
        }
        Action::Cancel => {
            println!("Aborted.");
            Ok(())
        }
        Action::Trash | Action::Hard => {
            let hard = matches!(action, Action::Hard);
            if hard && !opts.yes && !confirm_hard_delete(&plans)? {
                println!("Aborted.");
                return Ok(());
            }
            run_removal(&plans, hard)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Trash,
    Hard,
    DryRun,
    Cancel,
}

fn decide_action(opts: &UninstallOpts) -> Result<Action> {
    if opts.dry_run {
        return Ok(Action::DryRun);
    }
    if opts.yes {
        return Ok(if opts.hard { Action::Hard } else { Action::Trash });
    }
    let items = [
        "Move to Trash (recoverable)",
        "Dry-run (no changes)",
        "Hard delete (NOT recoverable)",
        "Cancel",
    ];
    let default_idx = if opts.hard { 2 } else { 0 };
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What to do?")
        .items(&items)
        .default(default_idx)
        .interact()
        .context("action menu failed")?;
    Ok(match idx {
        0 => Action::Trash,
        1 => Action::DryRun,
        2 => Action::Hard,
        _ => Action::Cancel,
    })
}

fn confirm_hard_delete(plans: &[Plan]) -> Result<bool> {
    let total: u64 = plans.iter().map(|p| p.total_size()).sum();
    let prompt = format!(
        "PERMANENTLY DELETE {} ({}) — this CANNOT be undone. Are you sure?",
        plural(plans.iter().map(|p| p.items.len()).sum::<usize>(), "item"),
        format_bytes(total)
    );
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(false)
        .interact()
        .context("hard-delete confirm prompt failed")?)
}

fn plural(n: usize, label: &str) -> String {
    if n == 1 {
        format!("{} {}", n, label)
    } else {
        format!("{} {}s", n, label)
    }
}

fn run_removal(plans: &[Plan], hard: bool) -> Result<()> {
    let mut failures: Vec<String> = Vec::new();
    for plan in plans {
        match execute_plan(plan, hard) {
            Ok(()) => println!("✓ removed {}", plan.app.name),
            Err(e) => {
                println!("✗ {}: {}", plan.app.name, e);
                failures.push(plan.app.name.clone());
            }
        }
    }
    if !failures.is_empty() {
        anyhow::bail!("failed to fully remove: {}", failures.join(", "));
    }
    Ok(())
}

// ---------- Data types ----------

#[derive(Debug, Clone)]
struct AppEntry {
    name: String,
    path: PathBuf,
    bundle_id: Option<String>,
    size: u64,
    last_used_days: Option<u64>,
}

#[derive(Debug)]
struct Plan {
    app: AppEntry,
    items: Vec<RemovalItem>,
    blocked: Option<String>,
}

#[derive(Debug)]
struct RemovalItem {
    path: PathBuf,
    size: u64,
    kind: ItemKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemKind {
    AppBundle,
    Leftover,
}

impl Plan {
    fn total_size(&self) -> u64 {
        self.items.iter().map(|i| i.size).sum()
    }
}

// ---------- App discovery ----------

fn list_applications() -> Result<Vec<AppEntry>> {
    let mut apps = Vec::new();
    let entries = fs::read_dir(APPLICATIONS_DIR)
        .with_context(|| format!("failed to read {}", APPLICATIONS_DIR))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "app").unwrap_or(false) {
            apps.push(load_app_entry(&path));
        }
    }
    Ok(apps)
}

fn load_app_entry(path: &Path) -> AppEntry {
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    AppEntry {
        name,
        path: path.to_path_buf(),
        bundle_id: read_bundle_id(path),
        size: dir_size(path),
        last_used_days: read_last_used_days(path),
    }
}

fn resolve_app_by_name(name: &str) -> Result<AppEntry> {
    let direct = PathBuf::from(APPLICATIONS_DIR).join(format!("{}.app", name));
    if direct.is_dir() {
        return Ok(load_app_entry(&direct));
    }
    // Case-insensitive fallback
    let entries = fs::read_dir(APPLICATIONS_DIR)
        .with_context(|| format!("failed to read {}", APPLICATIONS_DIR))?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().map(|e| e == "app").unwrap_or(false) {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                if stem.eq_ignore_ascii_case(name) {
                    return Ok(load_app_entry(&p));
                }
            }
        }
    }
    Err(anyhow!("no app named '{}' found in {}", name, APPLICATIONS_DIR))
}

fn read_bundle_id(app_path: &Path) -> Option<String> {
    let info = app_path.join("Contents/Info");
    let output = Command::new("defaults")
        .args(["read", info.to_str()?, "CFBundleIdentifier"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn read_last_used_days(app_path: &Path) -> Option<u64> {
    // mdls -name kMDItemLastUsedDate -raw <path>
    let output = Command::new("mdls")
        .args([
            "-name",
            "kMDItemLastUsedDate",
            "-raw",
            app_path.to_str()?,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8(output.stdout).ok()?.trim().to_string();
    parse_mdls_days_ago(&raw)
}

fn parse_mdls_days_ago(raw: &str) -> Option<u64> {
    if raw.is_empty() || raw == "(null)" {
        return None;
    }
    // Format: "2026-04-15 09:12:33 +0000"
    let date_part = raw.split_whitespace().next()?;
    let mut parts = date_part.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    // Days since civil epoch 1970-01-01 using Howard Hinnant's algorithm.
    let last_epoch_day = days_from_civil(year, month, day)?;
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let now_epoch_day = now_secs / 86_400;
    let diff = now_epoch_day - last_epoch_day;
    if diff < 0 { Some(0) } else { Some(diff as u64) }
}

fn days_from_civil(y: i64, m: i64, d: i64) -> Option<i64> {
    if m < 1 || m > 12 || d < 1 || d > 31 {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    let entries = match fs::read_dir(path) {
        Ok(it) => it,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_dir() {
            total = total.saturating_add(dir_size(&entry.path()));
        } else if meta.is_file() {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

// ---------- Sorting & picker ----------

fn sort_apps(mut apps: Vec<AppEntry>, sort: &SortBy) -> Vec<AppEntry> {
    match sort {
        SortBy::Size => apps.sort_by(|a, b| b.size.cmp(&a.size)),
        SortBy::Name => apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        SortBy::LastUsed => apps.sort_by(|a, b| {
            // Larger "days ago" = less recently used = surface first.
            // None (never tracked) ranks last.
            let ka = a.last_used_days.unwrap_or(0);
            let kb = b.last_used_days.unwrap_or(0);
            match (a.last_used_days.is_none(), b.last_used_days.is_none()) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => kb.cmp(&ka),
            }
        }),
    }
    apps
}

fn pick_interactive(opts: &UninstallOpts) -> Result<Vec<AppEntry>> {
    let apps = list_applications()?;
    let apps = sort_apps(apps, &opts.sort);

    if apps.is_empty() {
        return Ok(Vec::new());
    }

    let labels: Vec<String> = apps
        .iter()
        .map(|a| {
            let last_used = match a.last_used_days {
                Some(d) if d == 0 => "today".to_string(),
                Some(d) => format!("{}d ago", d),
                None => "never".to_string(),
            };
            format!(
                "{:<32} {:>10}    last used {}",
                truncate(&a.name, 32),
                format_bytes(a.size),
                last_used
            )
        })
        .collect();

    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select apps to uninstall (Space to toggle, Enter to confirm)")
        .items(&labels)
        .interact()
        .context("interactive picker failed")?;

    Ok(selected.into_iter().map(|i| apps[i].clone()).collect())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

// ---------- Planning ----------

fn build_plan(app: &AppEntry, opts: &UninstallOpts) -> Result<Plan> {
    let mut items: Vec<RemovalItem> = Vec::new();
    let mut blocked: Option<String> = None;

    if let Some(bid) = &app.bundle_id {
        if bid.starts_with(SYSTEM_BUNDLE_PREFIX) {
            blocked = Some(format!("system app ({}) — refuse", bid));
        }
    }

    if blocked.is_none() && is_brew_cask(&app.name) && !opts.force {
        blocked = Some(format!(
            "looks like a Homebrew cask — use `brew uninstall --cask {}` (or pass --force)",
            app.name.to_lowercase()
        ));
    }

    if !opts.leftovers_only {
        items.push(RemovalItem {
            path: app.path.clone(),
            size: app.size,
            kind: ItemKind::AppBundle,
        });
    }

    if !opts.shallow {
        let leftovers = find_leftovers(&app.name, app.bundle_id.as_deref());
        items.extend(leftovers);
    }

    Ok(Plan {
        app: app.clone(),
        items,
        blocked,
    })
}

fn is_brew_cask(name: &str) -> bool {
    let lower = name.to_lowercase().replace(' ', "-");
    let path = PathBuf::from("/opt/homebrew/Caskroom").join(&lower);
    path.is_dir()
}

fn find_leftovers(app_name: &str, bundle_id: Option<&str>) -> Vec<RemovalItem> {
    let mut items = Vec::new();
    let home = match std::env::var_os("HOME").map(PathBuf::from) {
        Some(h) => h,
        None => return items,
    };

    let mut candidates: Vec<PathBuf> = Vec::new();

    // Bundle-id-keyed locations
    if let Some(bid) = bundle_id {
        candidates.push(home.join("Library/Application Support").join(bid));
        candidates.push(home.join("Library/Caches").join(bid));
        candidates.push(home.join("Library/Preferences").join(format!("{}.plist", bid)));
        candidates.push(home.join("Library/Containers").join(bid));
        candidates.push(
            home.join("Library/Saved Application State")
                .join(format!("{}.savedState", bid)),
        );
        candidates.push(home.join("Library/HTTPStorages").join(bid));
        candidates.push(
            home.join("Library/HTTPStorages")
                .join(format!("{}.binarycookies", bid)),
        );
        candidates.push(home.join("Library/WebKit").join(bid));

        // LaunchAgents pattern <bundle_id>*.plist
        if let Ok(entries) = fs::read_dir(home.join("Library/LaunchAgents")) {
            for e in entries.flatten() {
                let p = e.path();
                if let Some(stem) = p.file_name().and_then(|s| s.to_str()) {
                    if stem.starts_with(bid) {
                        candidates.push(p);
                    }
                }
            }
        }
        // Group Containers — substring match on bundle id
        if let Ok(entries) = fs::read_dir(home.join("Library/Group Containers")) {
            for e in entries.flatten() {
                let p = e.path();
                if let Some(stem) = p.file_name().and_then(|s| s.to_str()) {
                    if stem.contains(bid) {
                        candidates.push(p);
                    }
                }
            }
        }
    }

    // Name-keyed fallback locations
    candidates.push(home.join("Library/Application Support").join(app_name));
    candidates.push(home.join("Library/Logs").join(app_name));
    candidates.push(home.join("Library/Caches").join(app_name));

    for path in candidates {
        if !path.exists() {
            continue;
        }
        let size = if path.is_dir() {
            dir_size(&path)
        } else {
            fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        };
        items.push(RemovalItem {
            path,
            size,
            kind: ItemKind::Leftover,
        });
    }
    items
}

// ---------- Reporting & confirm ----------

fn print_plans(plans: &[Plan], opts: &UninstallOpts) {
    println!(
        "Mode: {}",
        match (opts.shallow, opts.leftovers_only, opts.hard) {
            (_, _, true) => "rm -rf (NOT recoverable)",
            (true, _, _) => "shallow (only /Applications, move to Trash)",
            (_, true, _) => "leftovers only (only ~/Library, move to Trash)",
            _ => "full (move to Trash)",
        }
    );
    println!();

    let mut grand_total = 0u64;
    for plan in plans {
        println!("== {} ==", plan.app.name);
        if let Some(reason) = &plan.blocked {
            println!("  BLOCKED: {}", reason);
        }
        if let Some(bid) = &plan.app.bundle_id {
            println!("  bundle: {}", bid);
        }
        if plan.items.is_empty() {
            println!("  (nothing to remove)");
        }
        for item in &plan.items {
            let kind = match item.kind {
                ItemKind::AppBundle => "app  ",
                ItemKind::Leftover => "left.",
            };
            println!(
                "  {} {:>10}  {}",
                kind,
                format_bytes(item.size),
                item.path.display()
            );
        }
        let total = plan.total_size();
        println!("  → subtotal: {}", format_bytes(total));
        grand_total = grand_total.saturating_add(total);
        println!();
    }
    println!("Grand total: {}", format_bytes(grand_total));
}

// ---------- Execution ----------

fn execute_plan(plan: &Plan, hard: bool) -> Result<()> {
    for item in &plan.items {
        if hard {
            hard_remove(&item.path)?;
        } else {
            move_to_trash(&item.path)?;
        }
    }
    Ok(())
}

fn move_to_trash(path: &Path) -> Result<()> {
    let posix = path
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 path: {}", path.display()))?;
    let script = format!(
        r#"tell application "Finder" to delete (POSIX file "{}" as alias)"#,
        posix.replace('"', "\\\"")
    );
    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .with_context(|| format!("failed to spawn osascript for {}", path.display()))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!("osascript failed for {}: {}", path.display(), err));
    }
    Ok(())
}

fn hard_remove(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)
        .with_context(|| format!("stat failed: {}", path.display()))?;
    if meta.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!("remove_dir_all failed: {}", path.display()))?;
    } else {
        fs::remove_file(path)
            .with_context(|| format!("remove_file failed: {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mdls_handles_null_and_empty() {
        assert_eq!(parse_mdls_days_ago(""), None);
        assert_eq!(parse_mdls_days_ago("(null)"), None);
    }

    #[test]
    fn parse_mdls_extracts_days() {
        // 1970-01-01 → epoch day 0; today is many days later, must be >= 20000.
        let days = parse_mdls_days_ago("1970-01-01 00:00:00 +0000").unwrap();
        assert!(days >= 20_000);
    }

    #[test]
    fn truncate_is_safe() {
        assert_eq!(truncate("hi", 10), "hi");
        let t = truncate("supercalifragilisticexpialidocious", 10);
        assert_eq!(t.chars().count(), 10);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn sort_size_desc() {
        let apps = vec![mk("A", 100), mk("B", 300), mk("C", 200)];
        let s = sort_apps(apps, &SortBy::Size);
        assert_eq!(s[0].name, "B");
        assert_eq!(s[2].name, "A");
    }

    fn mk(name: &str, size: u64) -> AppEntry {
        AppEntry {
            name: name.to_string(),
            path: PathBuf::from(format!("/Applications/{}.app", name)),
            bundle_id: None,
            size,
            last_used_days: None,
        }
    }
}
