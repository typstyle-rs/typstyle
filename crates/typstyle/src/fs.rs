//! Helpers for normalizing and displaying file paths.

// Adapted from: https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/fs.rs

use std::path::{Path, PathBuf};

use path_absolutize::Absolutize;

/// Convert any path to an absolute path (based on the current working directory).
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    path.absolutize()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| path.to_path_buf())
}

/// Convert an absolute path to be relative to the current working directory.
pub fn relativize_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    path.strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .unwrap_or(path)
        .display()
        .to_string()
}
