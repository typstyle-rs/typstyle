//! Resolves the input paths to the files to format.
//!
//! Directories are walked with [`walkdir`]. Ignore-rule handling lives in
//! [`crate::gitignore`].

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use itertools::Itertools;
use walkdir::WalkDir;

use crate::{cli::FileSelectionArgs, fs as project_fs, gitignore::IgnoreWalk};

/// Resolves the input paths to the files to format.
///
/// Directories are searched recursively for `.typ` files. Entries whose name
/// starts with a dot are skipped, and the standard ignore files are consulted
/// unless the selection options opt out. Explicitly named files are always
/// kept, even when they are hidden, ignored, or use another extension.
/// Duplicates are removed, and the results are sorted if any directory was
/// expanded.
///
/// Ignore matching follows the ignore patterns of the repository; it never
/// consults Git's tracked-file index, so a tracked file matching an ignore
/// pattern is skipped during recursive discovery.
pub(crate) fn resolve_typst_files(
    input: &[PathBuf],
    selection: &FileSelectionArgs,
) -> Result<Vec<PathBuf>> {
    let mut roots = Vec::new();
    let mut files = Vec::new();
    for path in input.iter().map(project_fs::normalize_path).unique() {
        if path.is_dir() {
            roots.push(path);
        } else {
            files.push(path);
        }
    }

    let has_dir = !roots.is_empty();
    let respect = selection.respect_ignore_files();
    for root in &roots {
        collect_files(root, respect, &mut files)?;
    }
    if has_dir {
        files.sort_unstable();
    }
    Ok(files.into_iter().unique().collect())
}

/// Walks one root and appends the discovered `.typ` files.
fn collect_files(root: &Path, respect: bool, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut ignore = IgnoreWalk::new(root, respect)?;
    let mut walker = WalkDir::new(root).follow_links(false).into_iter();
    while let Some(entry) = walker.next() {
        let entry =
            entry.with_context(|| format!("failed to discover files under {}", root.display()))?;
        let depth = entry.depth();
        ignore.leave(depth);
        let is_dir = entry.file_type().is_dir();
        if depth > 0 {
            if is_hidden(entry.path()) {
                if is_dir {
                    walker.skip_current_dir();
                }
                continue;
            }
            if ignore.is_ignored(entry.path(), is_dir) {
                if is_dir {
                    walker.skip_current_dir();
                }
                continue;
            }
        }

        if entry.file_type().is_file() && entry.path().extension() == Some("typ".as_ref()) {
            files.push(entry.path().to_path_buf());
        }

        if is_dir {
            ignore.enter(entry.path(), depth)?;
        }
    }
    Ok(())
}

/// Returns whether the entry name starts with a dot.
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}
