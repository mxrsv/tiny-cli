//! Symlink-safe filesystem helpers.
//!
//! Uninstall's `dir_size` follows symlinks via `is_dir()` — we explicitly do
//! NOT do that here. Every metadata read uses `symlink_metadata`, walks never
//! descend into symlinked directories, and removal never calls
//! `fs::remove_dir_all`.

use std::fs;
use std::path::{Path, PathBuf};

/// Yields every entry under `root` without descending into symlinks. Symlink
/// entries themselves are yielded (so callers can remove them), but their
/// targets are not walked.
///
/// The walker silently skips entries whose metadata cannot be read; cleanup
/// continues on a best-effort basis.
#[allow(dead_code)]
pub fn walk_no_follow(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let meta = match fs::symlink_metadata(root) {
        Ok(m) => m,
        Err(_) => return out,
    };
    out.push(root.to_path_buf());
    if meta.file_type().is_dir() && !meta.file_type().is_symlink() {
        walk_inner(root, &mut out);
    }
    out
}

#[allow(dead_code)]
fn walk_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        out.push(path.clone());
        let ft = meta.file_type();
        if ft.is_dir() && !ft.is_symlink() {
            walk_inner(&path, out);
        }
    }
}

/// Walks `root` symlink-safe, calling `visit` for every directory entry.
/// `visit` returns whether to descend into that directory (ignored for
/// non-dir entries). The walker silently skips entries whose metadata
/// cannot be read.
///
/// Used by providers that need conditional descent (e.g. node_modules: stop
/// recursing once a `node_modules` dir is found, so we don't flag nested
/// transitive `node_modules`).
pub fn walk_with<F>(root: &Path, mut visit: F)
where
    F: FnMut(&Path, &fs::Metadata) -> bool,
{
    let meta = match fs::symlink_metadata(root) {
        Ok(m) => m,
        Err(_) => return,
    };
    if meta.file_type().is_symlink() {
        return;
    }
    if !meta.file_type().is_dir() {
        return;
    }
    let descend_root = visit(root, &meta);
    if descend_root {
        walk_with_inner(root, &mut visit);
    }
}

fn walk_with_inner<F>(dir: &Path, visit: &mut F)
where
    F: FnMut(&Path, &fs::Metadata) -> bool,
{
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let ft = meta.file_type();
        if ft.is_symlink() {
            continue;
        }
        let descend = visit(&path, &meta);
        if ft.is_dir() && descend {
            walk_with_inner(&path, visit);
        }
    }
}

/// True iff `path` is a real directory (not a symlink). Replacement for
/// `Path::is_dir()` which silently follows symlinks — providers must never
/// trust a symlinked target.
pub fn is_dir_safe(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_dir())
        .unwrap_or(false)
}

/// Sums the byte length of every file under `root`, never following symlinks.
/// Returns 0 if `root` does not exist.
pub fn dir_size_safe(root: &Path) -> u64 {
    let meta = match fs::symlink_metadata(root) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    let ft = meta.file_type();
    if ft.is_symlink() {
        return 0;
    }
    if ft.is_file() {
        return meta.len();
    }
    if !ft.is_dir() {
        return 0;
    }
    let mut total = 0u64;
    sum_dir(root, &mut total);
    total
}

fn sum_dir(dir: &Path, total: &mut u64) {
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let ft = meta.file_type();
        if ft.is_symlink() {
            continue;
        }
        if ft.is_file() {
            *total = total.saturating_add(meta.len());
        } else if ft.is_dir() {
            sum_dir(&path, total);
        }
    }
}

/// Recursively removes `root`, never following symlinks. Files and symlink
/// entries go through `fs::remove_file`; directories are removed bottom-up
/// via `fs::remove_dir`. Never calls `fs::remove_dir_all`.
pub fn remove_recursive_safe(root: &Path) -> std::io::Result<()> {
    let meta = match fs::symlink_metadata(root) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    let ft = meta.file_type();
    if ft.is_symlink() || ft.is_file() {
        return fs::remove_file(root);
    }
    if !ft.is_dir() {
        return Ok(());
    }
    remove_dir_contents(root)?;
    fs::remove_dir(root)
}

fn remove_dir_contents(dir: &Path) -> std::io::Result<()> {
    let entries = fs::read_dir(dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let ft = meta.file_type();
        if ft.is_symlink() || ft.is_file() {
            let _ = fs::remove_file(&path);
        } else if ft.is_dir() {
            remove_dir_contents(&path)?;
            let _ = fs::remove_dir(&path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn tempdir(label: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        let unique = format!(
            "tiny-clean-test-{}-{}",
            label,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        base.push(unique);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn write_file(p: &Path, bytes: &[u8]) {
        let mut f = File::create(p).unwrap();
        f.write_all(bytes).unwrap();
    }

    #[test]
    fn dir_size_counts_files_recursively() {
        let dir = tempdir("size");
        write_file(&dir.join("a"), b"hello"); // 5
        fs::create_dir(dir.join("sub")).unwrap();
        write_file(&dir.join("sub/b"), b"worldworld"); // 10
        assert_eq!(dir_size_safe(&dir), 15);
        let _ = remove_recursive_safe(&dir);
    }

    #[test]
    fn dir_size_zero_for_missing_path() {
        assert_eq!(dir_size_safe(Path::new("/tmp/__tiny_clean_missing__")), 0);
    }

    #[test]
    fn remove_safe_does_not_follow_symlinks() {
        let outside = tempdir("outside");
        write_file(&outside.join("keep"), b"keep me");

        let inside = tempdir("inside");
        write_file(&inside.join("file"), b"x");
        let link = inside.join("link-to-outside");
        std::os::unix::fs::symlink(&outside, &link).unwrap();

        assert!(outside.exists());
        assert!(link.exists());

        remove_recursive_safe(&inside).unwrap();

        assert!(!inside.exists());
        assert!(outside.exists(), "symlink target must NOT be deleted");
        assert!(outside.join("keep").exists());

        let _ = remove_recursive_safe(&outside);
    }

    #[test]
    fn dir_size_does_not_follow_symlinks() {
        let outside = tempdir("size-outside");
        write_file(&outside.join("big"), &vec![0u8; 1024]);

        let inside = tempdir("size-inside");
        write_file(&inside.join("small"), b"hi"); // 2
        let link = inside.join("link");
        std::os::unix::fs::symlink(&outside, &link).unwrap();

        assert_eq!(dir_size_safe(&inside), 2);

        let _ = remove_recursive_safe(&inside);
        let _ = remove_recursive_safe(&outside);
    }

    #[test]
    fn walk_no_follow_skips_symlink_targets() {
        let outside = tempdir("walk-outside");
        write_file(&outside.join("hidden"), b"x");

        let inside = tempdir("walk-inside");
        write_file(&inside.join("a"), b"a");
        let link = inside.join("link");
        std::os::unix::fs::symlink(&outside, &link).unwrap();

        let walked = walk_no_follow(&inside);
        let names: Vec<String> = walked
            .iter()
            .map(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default())
            .collect();
        assert!(names.iter().any(|n| n == "a"));
        assert!(names.iter().any(|n| n == "link"));
        assert!(!names.iter().any(|n| n == "hidden"));

        let _ = remove_recursive_safe(&inside);
        let _ = remove_recursive_safe(&outside);
    }
}
