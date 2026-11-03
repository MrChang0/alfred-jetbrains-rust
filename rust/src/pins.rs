//! Pinned projects and the shared on-disk format for hidden projects.

use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    io::{ErrorKind, Write},
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::projects::dedupe_key;

// Keep existing JSON strings readable. Only non-UTF-8 paths need a byte array.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum StoredPath {
    Text(String),
    Bytes(Vec<u8>),
}

pub fn load(path: &Path) -> Result<Vec<PathBuf>> {
    let text = match fs::read_to_string(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        result => result.with_context(|| format!("read {}", path.display()))?,
    };
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let paths: Vec<StoredPath> =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    Ok(paths
        .into_iter()
        .map(|stored| match stored {
            StoredPath::Text(text) => PathBuf::from(text),
            StoredPath::Bytes(bytes) => OsString::from_vec(bytes).into(),
        })
        .collect())
}

pub fn load_keys(path: &Path) -> Result<HashSet<PathBuf>> {
    Ok(load(path)?.iter().map(|path| dedupe_key(path)).collect())
}

fn save(path: &Path, projects: &[PathBuf]) -> Result<()> {
    let paths: Vec<_> = projects
        .iter()
        .map(|project| match project.to_str() {
            Some(text) => StoredPath::Text(text.to_owned()),
            None => StoredPath::Bytes(project.as_os_str().as_bytes().to_vec()),
        })
        .collect();
    let mut staged =
        tempfile::NamedTempFile::new_in(path.parent().context("missing state directory")?)?;
    serde_json::to_writer_pretty(&mut staged, &paths)?;
    staged.write_all(b"\n")?;
    if let Ok(metadata) = fs::metadata(path) {
        staged.as_file().set_permissions(metadata.permissions())?;
    }
    staged
        .persist(path)
        .with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

// A stable sibling lock covers the whole transaction, including across renames.
fn with_lock<T>(path: &Path, change: impl FnOnce(&Path) -> Result<T>) -> Result<T> {
    let path = if path.is_symlink() {
        fs::canonicalize(path).with_context(|| format!("resolve state file {}", path.display()))?
    } else {
        std::path::absolute(path)?
    };
    let parent = path.parent().context("missing state directory")?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let lock_path = path.with_added_extension("lock");
    let lock = fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .with_context(|| format!("open {}", lock_path.display()))?;
    lock.lock()
        .with_context(|| format!("lock {}", lock_path.display()))?;
    change(&path)
}

pub fn clear(path: &Path) -> Result<()> {
    with_lock(path, |path| save(path, &[]))
}

// Return the previous and new state so toggle can report the new state and
// pin/unpin can report whether anything changed.
fn set_pinned(path: &Path, project: &Path, desired: Option<bool>) -> Result<(bool, bool)> {
    let target = fs::canonicalize(project).or_else(|_| std::path::absolute(project))?;
    let key = dedupe_key(&target);
    with_lock(path, |path| {
        let mut projects = load(path)?;
        let matches = |stored: &PathBuf| dedupe_key(stored) == key;
        let previous = projects.iter().any(matches);
        let next = desired.unwrap_or(!previous);
        if next != previous {
            if next {
                projects.push(target);
            } else {
                projects.retain(|stored| !matches(stored));
            }
            save(path, &projects)?;
        }
        Ok((previous, next))
    })
}

pub fn pin(path: &Path, project: &Path) -> Result<bool> {
    let (previous, _) = set_pinned(path, project, Some(true))?;
    Ok(!previous)
}

pub fn unpin(path: &Path, project: &Path) -> Result<bool> {
    let (previous, _) = set_pinned(path, project, Some(false))?;
    Ok(previous)
}

pub fn toggle(path: &Path, project: &Path) -> Result<bool> {
    let (_, next) = set_pinned(path, project, None)?;
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn pin_and_unpin_round_trip() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let pins = dir.path().join("config/pins.json");

        assert!(pin(&pins, &project).unwrap());
        assert!(!pin(&pins, &project).unwrap());
        assert_eq!(load(&pins).unwrap().len(), 1);

        assert!(unpin(&pins, &project).unwrap());
        assert_eq!(load(&pins).unwrap().len(), 0);
    }

    #[test]
    fn unpin_reports_not_pinned() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let pins = dir.path().join("pins.json");

        assert!(!unpin(&pins, &project).unwrap());
        assert!(!pins.exists());
    }

    #[test]
    fn toggle_reports_state() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        fs::create_dir_all(&project).unwrap();
        let pins = dir.path().join("pins.json");

        assert!(toggle(&pins, &project).unwrap());
        assert!(!toggle(&pins, &project).unwrap());
    }
}
