use std::path::PathBuf;

use anyhow::Result;

use super::{execute_per_item, top_level_entries, CleanProvider};
use crate::commands::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "mail-attachments";
const LABEL: &str = "Mail attachments";
const APP: &str = "Mail";

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub struct MailAttachments;

impl CleanProvider for MailAttachments {
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
        home().map(|h| h.join("Library/Mail").is_dir()).unwrap_or(false)
    }
    fn discover(&self) -> Result<Vec<CleanItem>> {
        let h = match home() {
            Some(h) => h,
            None => return Ok(Vec::new()),
        };
        let mut items = Vec::new();
        for v_dir in v_dirs(&h.join("Library/Mail")) {
            let attachments = v_dir.join("MailData/Attachments");
            items.extend(top_level_entries(
                &attachments,
                ID,
                LABEL,
                RiskLevel::Review,
            ));
        }
        Ok(items)
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

/// Returns every direct child of `mail_root` whose name starts with `V`
/// followed by digits (Mail's per-version data dirs: `V8`, `V9`, ...).
pub fn v_dirs(mail_root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(mail_root) {
        Ok(it) => it,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.file_type().is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if name.starts_with('V')
            && name.len() > 1
            && name[1..].chars().all(|c| c.is_ascii_digit())
        {
            out.push(path);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::clean::providers::known_category_ids;
    use std::fs;

    fn tempdir(label: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        base.push(format!(
            "tiny-clean-mail-{}-{}",
            label,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn mail_attachments_id_in_known_categories() {
        assert!(known_category_ids().contains(&ID));
    }

    #[test]
    fn mail_gates_mail_app() {
        let p = MailAttachments;
        assert_eq!(p.requires_app_quit(), Some("Mail"));
    }

    #[test]
    fn v_dirs_matches_versioned_directories() {
        let root = tempdir("vdirs");
        fs::create_dir_all(root.join("V8")).unwrap();
        fs::create_dir_all(root.join("V10")).unwrap();
        fs::create_dir_all(root.join("Vacation")).unwrap(); // letter-suffix → must skip
        fs::create_dir_all(root.join("V")).unwrap(); // bare V → must skip
        fs::write(root.join("V99"), b"file").unwrap(); // file, not dir → must skip
        let mut found = v_dirs(&root);
        found.sort();
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|p| {
            let n = p.file_name().unwrap().to_str().unwrap();
            matches!(n, "V8" | "V10")
        }));
        let _ = fs::remove_dir_all(&root);
    }
}
