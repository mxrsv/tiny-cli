//! Move to Trash through Finder, so Finder's Put Back works. From the app
//! this needs Automation (Apple Events) permission for Finder; a denial is
//! its own error and never falls back to deletion.

use std::path::Path;
use std::time::Duration;

use crate::runner::{CommandRunner, RealRunner};

/// `osascript` program shared with the CLI's `move_to_trash`.
pub const FINDER_DELETE_SCRIPT: &str = "on run argv\n\
     tell application \"Finder\" to delete (POSIX file (item 1 of argv) as alias)\n\
     end run";

/// Absolute so a Finder-launched app does not depend on its minimal `PATH`.
const OSASCRIPT: &str = "/usr/bin/osascript";

/// Generous: the first call blocks on the Automation prompt until the user
/// answers, and Finder may take long to move a large directory.
const TRASH_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Apple Events error for "not authorized to send Apple events".
const AUTOMATION_DENIED: &str = "-1743";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TrashError {
    #[error("automation permission denied: {0}")]
    AutomationDenied(String),
    #[error("move to Trash failed: {0}")]
    Failed(String),
}

pub trait Trash: Send + Sync {
    /// Moves `path` to the user's Trash. On `Err` the source is left as is.
    fn move_to_trash(&self, path: &Path) -> Result<(), TrashError>;
}

/// The default route: Finder via `osascript`.
pub struct FinderTrash;

impl Trash for FinderTrash {
    fn move_to_trash(&self, path: &Path) -> Result<(), TrashError> {
        let posix = path
            .to_str()
            .ok_or_else(|| TrashError::Failed(format!("non-UTF-8 path: {}", path.display())))?;
        let outcome = RealRunner
            .output(
                OSASCRIPT,
                &["-e", FINDER_DELETE_SCRIPT, posix],
                TRASH_TIMEOUT,
            )
            .map_err(|e| TrashError::Failed(e.to_string()))?;
        if outcome.success() {
            return Ok(());
        }
        Err(classify_failure(outcome.stderr.trim()))
    }
}

fn classify_failure(stderr: &str) -> TrashError {
    if stderr.contains(AUTOMATION_DENIED) {
        TrashError::AutomationDenied(stderr.to_string())
    } else {
        TrashError::Failed(stderr.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_events_denial_is_its_own_error() {
        let denied = "execution error: Not authorized to send Apple events to Finder. (-1743)";
        assert!(matches!(
            classify_failure(denied),
            TrashError::AutomationDenied(_)
        ));
        let missing = "execution error: Finder got an error: Can’t get alias. (-1728)";
        assert!(matches!(classify_failure(missing), TrashError::Failed(_)));
    }
}
