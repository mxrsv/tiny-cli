use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::error::Result;

use super::{dir_roots_as_items, execute_per_item, CleanProvider};
use crate::clean::fs_safe::{is_dir_safe, list_children};
use crate::clean::scan_context::ScanContext;
use crate::clean::types::{CleanItem, ExecAction, ExecReport, RiskLevel};

const ID: &str = "macos-installers";
const LABEL: &str = "macOS installers";

const APPLICATIONS: &str = "/Applications";
const PREFIX: &str = "Install macOS ";
/// Process names of a running installer.
const INSTALLER_PROCESSES: &[&str] = &["InstallAssistant", "InstallAssistant_springboard"];

fn is_installer(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with(PREFIX) && n.ends_with(".app"))
}

/// Whether the current user may write `path` (`access(2)`, so ACLs and the
/// admin group count).
fn writable(path: &Path) -> bool {
    let Ok(c) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    // SAFETY: `c` is a valid NUL-terminated string for the whole call.
    unsafe { libc::access(c.as_ptr(), libc::W_OK) == 0 }
}

pub struct MacosInstallers;

impl MacosInstallers {
    pub(crate) fn discover_in(&self, ctx: &ScanContext<'_>, applications: &Path) -> Vec<CleanItem> {
        let roots: Vec<_> = list_children(applications, ctx)
            .into_iter()
            .filter(|p| is_installer(p) && is_dir_safe(p))
            .collect();
        dir_roots_as_items(ctx, &roots, ID, LABEL, RiskLevel::Review)
    }
}

impl CleanProvider for MacosInstallers {
    fn id(&self) -> &'static str {
        ID
    }
    fn label(&self) -> &'static str {
        LABEL
    }
    fn inclusion_reason(&self) -> String {
        "macOS installer apps left in Applications; the App Store downloads them again".into()
    }
    fn risk(&self) -> RiskLevel {
        RiskLevel::Review
    }
    fn desktop_trash_paths(&self) -> bool {
        true
    }
    fn quit_apps(&self) -> Vec<&'static str> {
        INSTALLER_PROCESSES.to_vec()
    }
    /// Refuses with a reason instead of letting Finder ask for an
    /// administrator password: installers fetched by `softwareupdate` are
    /// owned by root.
    fn check_item(&self, path: &Path) -> std::result::Result<(), String> {
        if !is_installer(path) {
            return Err(format!("{} is not a macOS installer", path.display()));
        }
        let parent = path.parent().unwrap_or(path);
        if writable(path) && writable(parent) {
            Ok(())
        } else {
            Err(format!(
                "moving {} needs administrator rights; drag it to the Trash in Finder",
                path.display()
            ))
        }
    }
    fn discover(&self, ctx: &ScanContext<'_>) -> Result<Vec<CleanItem>> {
        Ok(self.discover_in(ctx, Path::new(APPLICATIONS)))
    }
    fn execute(&self, items: &[CleanItem], action: ExecAction) -> Result<ExecReport> {
        execute_per_item(items, action, ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::providers::test_home::TestHome;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn offers_only_installer_bundles() {
        let apps = TestHome::new("macos-installers");
        apps.file(
            "Install macOS Sonoma.app/Contents/SharedSupport/SharedSupport.dmg",
            9,
        );
        apps.dir("Install macOS Ventura.app/Contents");
        apps.dir("Safari.app/Contents");
        apps.dir("Install Office.app/Contents");
        apps.file("Install macOS Notes.txt", 1);
        apps.symlink("Install macOS Link.app", "Safari.app");
        let items = MacosInstallers.discover_in(&ScanContext::unchecked(), apps.path());
        let mut found = apps.relative(&items);
        found.sort();
        assert_eq!(
            found,
            ["Install macOS Sonoma.app", "Install macOS Ventura.app"]
        );
    }

    #[test]
    fn refuses_an_installer_the_user_cannot_move() {
        let apps = TestHome::new("macos-installers-perm");
        apps.dir("Install macOS Sonoma.app/Contents");
        let app = apps.path().join("Install macOS Sonoma.app");
        assert!(MacosInstallers.check_item(&app).is_ok());
        assert!(MacosInstallers
            .check_item(&apps.path().join("Safari.app"))
            .is_err());
        // root ignores mode bits, so the refusal is only observable as a user.
        if unsafe { libc::geteuid() } != 0 {
            std::fs::set_permissions(apps.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
            let refused = MacosInstallers.check_item(&app).unwrap_err();
            std::fs::set_permissions(apps.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
            assert!(refused.contains("administrator"), "{refused}");
        }
    }
}
