use crate::error::{ErrorPayload, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    len: u64,
    modified: SystemTime,
    descendants: u64,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}
pub fn fingerprint(path: &Path) -> Result<Fingerprint> {
    tiny_core::space_lens::validate_path(path)?;
    let meta = fs::symlink_metadata(path)?;
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    Ok(Fingerprint {
        len: meta.len(),
        modified: meta.modified()?,
        descendants: directory_digest(path, &meta)?,
        #[cfg(unix)]
        device: meta.dev(),
        #[cfg(unix)]
        inode: meta.ino(),
    })
}
/// Include descendant metadata so an edited file inside an unchanged directory invalidates its preview.
fn directory_digest(path: &Path, metadata: &fs::Metadata) -> Result<u64> {
    use std::hash::{Hash, Hasher};
    let mut digest = std::collections::hash_map::DefaultHasher::new();
    if !metadata.is_dir() {
        return Ok(0);
    }
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let child = entry.path();
            let meta = fs::symlink_metadata(&child)?;
            child.hash(&mut digest);
            meta.len().hash(&mut digest);
            meta.modified()?.hash(&mut digest);
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                meta.dev().hash(&mut digest);
                meta.ino().hash(&mut digest);
            }
            if meta.is_dir() && !meta.file_type().is_symlink() {
                pending.push(child);
            }
        }
    }
    Ok(digest.finish())
}

pub trait Trash: Send + Sync {
    fn move_path(&self, path: &Path) -> Result<()>;
}
pub struct NativeTrash;
impl Trash for NativeTrash {
    fn move_path(&self, path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            trash::delete(path).map_err(|e| ErrorPayload::new("trash_failed", e))
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = path;
            Err(ErrorPayload::new(
                "unsupported",
                "macOS Trash is unavailable.",
            ))
        }
    }
}
/// Only a rename on the same volume is accepted as fallback: a failed copy can never lose the original.
pub fn move_recoverable(
    path: &Path,
    destination: &Path,
    expected: &Fingerprint,
    trash: &dyn Trash,
) -> Result<&'static str> {
    if fingerprint(path)? != *expected {
        return Err(ErrorPayload::new(
            "stale_preview",
            "This path changed after preview. Scan and review it again.",
        ));
    }
    if trash.move_path(path).is_ok() {
        return Ok("trash");
    }
    // Recheck after a failed Trash operation before attempting a fallback.
    if fingerprint(path)? != *expected {
        return Err(ErrorPayload::new(
            "stale_preview",
            "Path changed while attempting Trash.",
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| ErrorPayload::new("invalid_path", "Missing quarantine directory."))?;
    fs::create_dir_all(parent)?;
    tiny_core::space_lens::validate_path(parent)?;
    if fs::symlink_metadata(destination).is_ok() {
        return Err(ErrorPayload::new(
            "destination_exists",
            "Quarantine destination already exists.",
        ));
    }
    fs::rename(path, destination).map_err(|e| {
        ErrorPayload::new(
            "quarantine_failed",
            format!(
                "Trash and quarantine could not move this path; the original is preserved. {e}"
            ),
        )
    })?;
    Ok("quarantine")
}
pub fn restore(stored: &Path, original: &Path, quarantine_root: &Path) -> Result<()> {
    if !stored.starts_with(quarantine_root) {
        return Err(ErrorPayload::new(
            "invalid_restore",
            "Path is outside this app's quarantine.",
        ));
    }
    tiny_core::space_lens::validate_path(stored)?;
    if fs::symlink_metadata(original).is_ok() {
        return Err(ErrorPayload::new(
            "restore_conflict",
            "The original path exists. Move it aside before restoring.",
        ));
    }
    let parent = original
        .parent()
        .ok_or_else(|| ErrorPayload::new("invalid_restore", "Missing original parent."))?;
    // Do not recreate missing parents through a symlink or restore into an unexpected folder.
    tiny_core::space_lens::validate_path(parent)?;
    fs::rename(stored, original)?;
    Ok(())
}
pub fn quarantine_destination(root: &Path, id: &str) -> PathBuf {
    root.join(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FailedTrash;
    impl Trash for FailedTrash {
        fn move_path(&self, _: &Path) -> Result<()> {
            Err(ErrorPayload::new("trash_failed", "test"))
        }
    }
    #[test]
    fn fallback_restores_and_never_overwrites_an_existing_original() {
        let dir = crate::test_directory();
        let original = dir.path().join("cache");
        let quarantine = dir.path().join("quarantine");
        let stored = quarantine.join("entry");
        fs::write(&original, b"keep").unwrap();
        let before = fingerprint(&original).unwrap();
        assert_eq!(
            move_recoverable(&original, &stored, &before, &FailedTrash).unwrap(),
            "quarantine"
        );
        assert!(!original.exists());
        fs::write(&original, b"new").unwrap();
        assert_eq!(
            restore(&stored, &original, &quarantine).unwrap_err().code,
            "restore_conflict"
        );
        tiny_core::clean::fs_safe::remove_recursive_safe(&original).unwrap();
        restore(&stored, &original, &quarantine).unwrap();
        assert_eq!(fs::read(&original).unwrap(), b"keep");
    }
    #[test]
    fn descendant_edits_invalidate_a_directory_preview() {
        let dir = crate::test_directory();
        let cache = dir.path().join("cache");
        fs::create_dir(&cache).unwrap();
        let file = cache.join("item");
        fs::write(&file, b"old").unwrap();
        let before = fingerprint(&cache).unwrap();
        fs::write(&file, b"a longer replacement").unwrap();
        assert_eq!(
            move_recoverable(&cache, &dir.path().join("q"), &before, &FailedTrash)
                .unwrap_err()
                .code,
            "stale_preview"
        );
        assert!(file.exists());
    }
    #[test]
    fn changed_path_and_symlinked_ancestor_cannot_be_moved() {
        let dir = crate::test_directory();
        let path = dir.path().join("cache");
        fs::write(&path, b"a").unwrap();
        let before = fingerprint(&path).unwrap();
        fs::write(&path, b"longer").unwrap();
        assert_eq!(
            move_recoverable(&path, &dir.path().join("q"), &before, &FailedTrash)
                .unwrap_err()
                .code,
            "stale_preview"
        );
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(dir.path(), &link).unwrap();
        assert!(fingerprint(&link.join("cache")).is_err());
        assert!(path.exists());
    }
}
