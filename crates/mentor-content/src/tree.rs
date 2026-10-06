//! Checks the files of a package before anything reads them.
//!
//! A package comes from a Git repository its platform does not control. Before the compiler opens a file,
//! the whole directory is walked and refused when it holds anything other than ordinary files and folders,
//! or more than the limits allow. The rules are those of `doc/course-packages.md` §6; a fetched repository
//! is checked the same way as an author's working directory.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::error::{ContentError, Result};

/// What a package directory may hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    /// Files in the whole package.
    pub files: usize,
    /// Size of one file, in bytes.
    pub file_bytes: u64,
    /// Size of one file the compiler parses (`.md`, `.yml`, `.yaml`, `.json`), in bytes.
    pub source_bytes: u64,
    /// Size of all files together, in bytes.
    pub total_bytes: u64,
    /// Folders nested under the package root.
    pub depth: usize,
    /// Characters in one file or folder name.
    pub name_chars: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { files: 5_000, file_bytes: 8 << 20, source_bytes: 1 << 20, total_bytes: 64 << 20, depth: 16, name_chars: 100 }
    }
}

/// What a checked package directory holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct TreeSummary {
    pub files: usize,
    pub bytes: u64,
}

/// Extensions of the files the compiler parses.
const SOURCE_EXTENSIONS: [&str; 4] = ["md", "yml", "yaml", "json"];

/// Only these characters, so that a name means the same on every file system, in a URL and in a shell.
fn name_is_plain(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) && !name.starts_with('-')
}

fn walk(directory: &Path, depth: usize, limits: &Limits, summary: &mut TreeSummary) -> Result<()> {
    let unreadable = |err: std::io::Error| ContentError::new(directory, format!("cannot be read: {err}"));
    let mut entries = fs::read_dir(directory).map_err(unreadable)?.collect::<std::io::Result<Vec<_>>>().map_err(unreadable)?;
    // A fixed order, so that the same package always reports the same first error.
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let fail = |message: String| Err(ContentError::new(&path, message));
        let name = entry.file_name();
        let Some(name) = name.to_str().filter(|name| name_is_plain(name)) else {
            return fail("file and folder names hold only letters, digits, `.`, `_` and `-`, and do not start with `-`".into());
        };
        if name.chars().count() > limits.name_chars {
            return fail(format!("the name is longer than {} characters", limits.name_chars));
        }
        // An author's working directory is a Git repository; what Git keeps there is not part of the package.
        if depth == 0 && name == ".git" {
            continue;
        }
        // Never follows a link: `symlink_metadata` describes the link itself.
        let metadata = fs::symlink_metadata(&path).map_err(|err| ContentError::new(&path, format!("cannot be read: {err}")))?;
        let kind = metadata.file_type();
        if kind.is_symlink() {
            return fail("symbolic links are not allowed in a package".into());
        }
        if kind.is_dir() {
            if depth + 1 > limits.depth {
                return fail(format!("folders are nested more than {} levels deep", limits.depth));
            }
            walk(&path, depth + 1, limits, summary)?;
            continue;
        }
        if !kind.is_file() {
            return fail("only ordinary files and folders are allowed in a package".into());
        }
        if name == ".gitmodules" {
            return fail("Git submodules are not allowed in a package".into());
        }
        let size = metadata.len();
        let is_source = path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext));
        let allowed = if is_source { limits.source_bytes } else { limits.file_bytes };
        if size > allowed {
            return fail(format!("the file is {size} bytes, the limit is {allowed}"));
        }
        // Small enough to read here: the size was just checked.
        if name == ".gitattributes" && fs::read_to_string(&path).is_ok_and(|text| text.contains("filter=")) {
            return fail("Git filters (`filter=`, used by Git LFS) are not allowed in a package".into());
        }
        summary.files += 1;
        summary.bytes += size;
        if summary.files > limits.files {
            return Err(ContentError::new(directory, format!("the package holds more than {} files", limits.files)));
        }
        if summary.bytes > limits.total_bytes {
            return Err(ContentError::new(directory, format!("the package is larger than {} bytes", limits.total_bytes)));
        }
    }
    Ok(())
}

/// Walks `directory` without following links and refuses what a package may not hold.
pub fn verify_tree(directory: &Path, limits: &Limits) -> Result<TreeSummary> {
    let mut summary = TreeSummary::default();
    walk(directory, 0, limits, &mut summary)?;
    Ok(summary)
}
