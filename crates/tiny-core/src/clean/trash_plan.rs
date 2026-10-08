//! A reviewed selection of discovered paths for the desktop's Move to Trash:
//! what each path looked like at discovery, and which selected paths overlap.

use std::path::{Path, PathBuf};

use super::fs_safe::PathFingerprint;
use super::types::CleanItem;

/// One selected path with the trusted discovery-time data execution checks
/// it against.
#[derive(Debug, Clone)]
pub struct PlannedItem {
    pub item: CleanItem,
    pub fingerprint: PathFingerprint,
    /// Directories discovery listed or walked for the item's category;
    /// execution acts only on paths inside one of them.
    pub roots: Vec<PathBuf>,
    /// Selected items merged into this one (the same path, or paths inside
    /// it). Moving this item moves them too, so their providers' guards and
    /// app gates apply as well.
    pub covers: Vec<CleanItem>,
}

/// Why a selected path is left out of the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlap {
    /// The same path is already selected (another category found it too).
    Duplicate { kept: usize },
    /// An ancestor is already selected and moves this path with it.
    Inside { kept: usize },
}

/// Splits `entries` into the paths to act on (sorted by path) and the ones
/// another kept entry already covers; `kept` indexes the first list. This
/// keeps one path from being moved twice and its bytes from being counted
/// twice.
pub fn partition_overlaps<T>(
    mut entries: Vec<T>,
    path: impl Fn(&T) -> &Path,
) -> (Vec<T>, Vec<(T, Overlap)>) {
    // Component order puts every descendant right after its ancestor.
    entries.sort_by(|a, b| path(a).cmp(path(b)));
    let mut kept: Vec<T> = Vec::new();
    let mut covered = Vec::new();
    for entry in entries {
        let overlap = kept.last().and_then(|last| {
            let index = kept.len() - 1;
            if path(&entry) == path(last) {
                Some(Overlap::Duplicate { kept: index })
            } else if path(&entry).starts_with(path(last)) {
                Some(Overlap::Inside { kept: index })
            } else {
                None
            }
        });
        match overlap {
            Some(overlap) => covered.push((entry, overlap)),
            None => kept.push(entry),
        }
    }
    (kept, covered)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(paths: &[&str]) -> (Vec<PathBuf>, Vec<(PathBuf, Overlap)>) {
        let entries = paths.iter().map(PathBuf::from).collect();
        partition_overlaps(entries, |p: &PathBuf| p.as_path())
    }

    #[test]
    fn duplicates_and_descendants_are_covered_by_the_kept_path() {
        let (kept, covered) = split(&[
            "/c/Caches/com.apple.Safari",
            "/c/Caches",
            "/c/Caches-other",
            "/c/Caches/go-build",
            "/c/Caches",
        ]);
        assert_eq!(
            kept,
            vec![PathBuf::from("/c/Caches"), PathBuf::from("/c/Caches-other")]
        );
        assert_eq!(
            covered,
            vec![
                (PathBuf::from("/c/Caches"), Overlap::Duplicate { kept: 0 }),
                (
                    PathBuf::from("/c/Caches/com.apple.Safari"),
                    Overlap::Inside { kept: 0 }
                ),
                (
                    PathBuf::from("/c/Caches/go-build"),
                    Overlap::Inside { kept: 0 }
                ),
            ]
        );
    }

    #[test]
    fn a_name_prefix_is_not_an_ancestor() {
        let (kept, covered) = split(&["/a/node", "/a/node_modules"]);
        assert_eq!(kept.len(), 2);
        assert!(covered.is_empty());
    }
}
