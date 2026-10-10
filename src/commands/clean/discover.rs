//! Build the active provider list from CLI options and run discovery.
//!
//! Risk gating:
//! - `--category` overrides risk filtering. Naming a category counts as
//!   explicit consent; `--include-*` flags are ignored when `--category` is
//!   set.
//! - Otherwise: Safe by default; `--include-review` adds Review;
//!   `--include-destructive` adds Destructive.

use anyhow::Result;
use std::collections::BTreeMap;

use super::process::{ProcessChecker, PgrepChecker};
use super::providers::{all_providers, CleanProvider};
use super::types::{CleanItem, RiskLevel};
use crate::cli::CleanOpts;

pub struct CategoryGroup {
    pub id: String,
    pub label: String,
    pub risk: RiskLevel,
    pub items: Vec<CleanItem>,
    pub total_size: u64,
}

pub struct DiscoveryReport {
    pub groups: Vec<CategoryGroup>,
    pub skipped_running: Vec<(String, String)>, // (category_id, app)
    /// Categories whose scan failed, with the error. One failing category
    /// never hides the others.
    pub failed: Vec<(String, String)>, // (category_id, error)
}

/// Returns the providers selected by the CLI options. Filtering rules from
/// the plan flag matrix.
pub fn select_providers(opts: &CleanOpts) -> Vec<Box<dyn CleanProvider>> {
    let all = all_providers(opts);
    if !opts.category.is_empty() {
        return all
            .into_iter()
            .filter(|p| opts.category.iter().any(|c| c == p.id()))
            .collect();
    }
    all.into_iter()
        .filter(|p| match p.risk() {
            RiskLevel::Safe => true,
            RiskLevel::Review => opts.include_review,
            RiskLevel::Destructive => opts.include_destructive,
        })
        .collect()
}

pub fn discover(opts: &CleanOpts) -> Result<DiscoveryReport> {
    discover_with_checker(opts, &PgrepChecker)
}

pub fn discover_with_checker(
    opts: &CleanOpts,
    checker: &dyn ProcessChecker,
) -> Result<DiscoveryReport> {
    discover_providers(select_providers(opts), checker)
}

fn discover_providers(
    providers: Vec<Box<dyn CleanProvider>>,
    checker: &dyn ProcessChecker,
) -> Result<DiscoveryReport> {
    let mut groups_by_id: BTreeMap<String, CategoryGroup> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    for provider in providers {
        if !provider.available() {
            continue;
        }
        if let Some(app) = provider.requires_app_quit() {
            if checker.is_running(app) {
                skipped.push((provider.id().to_string(), app.to_string()));
                continue;
            }
        }
        let items = match provider.discover() {
            Ok(items) => items,
            Err(e) => {
                failed.push((provider.id().to_string(), format!("{:#}", e)));
                continue;
            }
        };
        if items.is_empty() {
            continue;
        }
        let id = provider.id().to_string();
        if !groups_by_id.contains_key(&id) {
            order.push(id.clone());
        }
        let entry = groups_by_id.entry(id.clone()).or_insert_with(|| CategoryGroup {
            id: id.clone(),
            label: provider.label().to_string(),
            risk: provider.risk(),
            items: Vec::new(),
            total_size: 0,
        });
        for item in items {
            entry.total_size = entry.total_size.saturating_add(item.size);
            entry.items.push(item);
        }
    }

    let groups: Vec<CategoryGroup> = order
        .into_iter()
        .filter_map(|id| groups_by_id.remove(&id))
        .collect();

    Ok(DiscoveryReport {
        groups,
        skipped_running: skipped,
        failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CleanOpts;

    fn opts() -> CleanOpts {
        CleanOpts {
            dry_run: false,
            yes: false,
            hard: false,
            category: Vec::new(),
            include_review: false,
            include_destructive: false,
            review_paths: false,
            idle_days: 30,
        }
    }

    #[test]
    fn default_only_safe_providers() {
        let providers = select_providers(&opts());
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        assert!(ids.contains(&"user-logs"));
        assert!(ids.contains(&"xcode-derived"));
        assert!(!ids.contains(&"user-caches"));
        assert!(!ids.contains(&"trash"));
    }

    #[test]
    fn include_review_adds_review() {
        let mut o = opts();
        o.include_review = true;
        let providers = select_providers(&o);
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        assert!(ids.contains(&"user-caches"));
        assert!(ids.contains(&"cargo"));
        assert!(!ids.contains(&"trash"));
    }

    #[test]
    fn include_destructive_adds_trash() {
        let mut o = opts();
        o.include_destructive = true;
        let providers = select_providers(&o);
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        assert!(ids.contains(&"trash"));
    }

    #[test]
    fn category_overrides_risk_filtering() {
        let mut o = opts();
        o.category = vec!["trash".into()];
        let providers = select_providers(&o);
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        assert_eq!(ids, vec!["trash"]);
    }

    #[test]
    fn category_can_pick_review_without_flag() {
        let mut o = opts();
        o.category = vec!["cargo".into()];
        let providers = select_providers(&o);
        let ids: Vec<&str> = providers.iter().map(|p| p.id()).collect();
        assert_eq!(ids, vec!["cargo"]);
    }

    #[test]
    fn running_xcode_skips_xcode_categories_at_discovery() {
        use crate::commands::clean::process::test_support::MockChecker;
        let mut o = opts();
        o.category = vec!["xcode-derived".into(), "xcode-archives".into()];
        let mock = MockChecker::with_running(["Xcode"]);
        let report = discover_with_checker(&o, &mock).unwrap();
        // Both Xcode categories must be skipped, not discovered.
        assert!(
            report.groups.iter().all(|g| g.id != "xcode-derived"),
            "xcode-derived should have been skipped"
        );
        assert!(
            report.groups.iter().all(|g| g.id != "xcode-archives"),
            "xcode-archives should have been skipped"
        );
        // And the skip must be reported.
        let skipped_apps: Vec<&str> = report
            .skipped_running
            .iter()
            .map(|(_, app)| app.as_str())
            .collect();
        assert!(skipped_apps.iter().all(|a| *a == "Xcode"));
        assert!(report.skipped_running.iter().any(|(id, _)| id == "xcode-derived"));
        assert!(report.skipped_running.iter().any(|(id, _)| id == "xcode-archives"));
    }
    #[test]
    fn a_failing_category_is_reported_and_others_still_scan() {
        use crate::commands::clean::process::test_support::MockChecker;
        use crate::commands::clean::types::{ExecAction, ExecReport};
        use std::path::PathBuf;

        struct Fixed(&'static str, Option<&'static str>);
        impl CleanProvider for Fixed {
            fn id(&self) -> &'static str {
                self.0
            }
            fn label(&self) -> &'static str {
                self.0
            }
            fn risk(&self) -> RiskLevel {
                RiskLevel::Safe
            }
            fn discover(&self) -> Result<Vec<CleanItem>> {
                match self.1 {
                    Some(err) => Err(anyhow::anyhow!(err)),
                    None => Ok(vec![CleanItem {
                        category_id: self.0.into(),
                        category_label: self.0.into(),
                        path: PathBuf::from("/tmp/x"),
                        size: 1,
                        risk: RiskLevel::Safe,
                    }]),
                }
            }
            fn execute(&self, _: &[CleanItem], _: ExecAction) -> Result<ExecReport> {
                Ok(ExecReport::default())
            }
        }

        let providers: Vec<Box<dyn CleanProvider>> = vec![
            Box::new(Fixed("broken", Some("cannot read /x: permission denied"))),
            Box::new(Fixed("fine", None)),
        ];
        let report = discover_providers(providers, &MockChecker::none()).unwrap();
        let ids: Vec<&str> = report.groups.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["fine"]);
        assert_eq!(
            report.failed,
            vec![(
                "broken".to_string(),
                "cannot read /x: permission denied".to_string()
            )]
        );
    }
}
