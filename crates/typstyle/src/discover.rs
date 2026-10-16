//! Resolves the input paths to the files to format.
//!
//! Directories are walked with [`walkdir`], and the standard ignore files are
//! matched with [`gix_ignore`]: `.ignore`, `.gitignore`, Git's local excludes,
//! and the global Git excludes configuration.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use bstr::BStr;
use gix_ignore::{Search, glob::pattern::Case, search::Ignore};
use itertools::Itertools;
use walkdir::WalkDir;

use crate::{cli::FileSelectionArgs, fs as project_fs};

/// The ignore rules that apply while walking one directory tree.
struct IgnoreContext {
    /// The layered ignore patterns, in ascending precedence.
    search: Search,
    /// The directory all matched paths are relative to.
    anchor: PathBuf,
    /// Whether the walk root is inside a Git repository.
    in_git_repo: bool,
}

impl IgnoreContext {
    /// Collects the rules that apply to a walk rooted at `root`.
    fn new(root: &Path, respect: bool) -> Result<Self> {
        let git_root = find_git_root(root);
        let anchor = git_root.clone().unwrap_or_else(|| root.to_path_buf());
        let mut context = Self {
            search: Search::default(),
            anchor,
            in_git_repo: git_root.is_some(),
        };
        if !respect {
            return Ok(context);
        }
        let Some(git_root) = git_root else {
            return Ok(context);
        };

        if let Some(excludes) = global_excludes_file() {
            add_rule_file(&mut context.search, &excludes, None)?;
        }
        add_rule_file(
            &mut context.search,
            &git_root.join(".git").join("info").join("exclude"),
            None,
        )?;

        if root != git_root {
            let mut dirs = Vec::new();
            let mut dir = root.parent();
            while let Some(current) = dir {
                if !current.starts_with(&git_root) {
                    break;
                }
                dirs.push(current.to_path_buf());
                if current == git_root {
                    break;
                }
                dir = current.parent();
            }
            for dir in dirs.iter().rev() {
                add_directory_rules(&mut context.search, dir, &context.anchor, true)?;
            }
        }
        Ok(context)
    }

    /// Loads the rules of `dir` for the entries below it.
    fn push_directory(&mut self, dir: &Path) -> Result<()> {
        add_directory_rules(&mut self.search, dir, &self.anchor, self.in_git_repo)
    }

    /// Returns whether the last matching rule excludes `path`.
    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let Ok(relative) = path.strip_prefix(&self.anchor) else {
            return false;
        };
        let relative = to_slash(relative);
        if relative.is_empty() {
            return false;
        }
        self.search
            .pattern_matching_relative_path(
                BStr::new(relative.as_bytes()),
                Some(is_dir),
                Case::Sensitive,
            )
            .is_some_and(|matched| !matched.pattern.is_negative())
    }
}

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
    let mut context = IgnoreContext::new(root, respect)?;
    let mut walker = WalkDir::new(root).follow_links(false).into_iter();
    while let Some(entry) = walker.next() {
        let entry =
            entry.with_context(|| format!("failed to discover files under {}", root.display()))?;
        let depth = entry.depth();
        let is_dir = entry.file_type().is_dir();
        if depth > 0 {
            if is_hidden(entry.path()) {
                if is_dir {
                    walker.skip_current_dir();
                }
                continue;
            }
            if respect && context.is_ignored(entry.path(), is_dir) {
                if is_dir {
                    walker.skip_current_dir();
                }
                continue;
            }
        }

        if entry.file_type().is_file() && entry.path().extension() == Some("typ".as_ref()) {
            files.push(entry.path().to_path_buf());
        }

        if is_dir && respect {
            context.push_directory(entry.path())?;
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

/// Returns the nearest ancestor that contains a `.git` entry.
fn find_git_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Appends the `.gitignore` and `.ignore` rules of `dir`.
fn add_directory_rules(
    search: &mut Search,
    dir: &Path,
    anchor: &Path,
    in_git_repo: bool,
) -> Result<()> {
    if in_git_repo {
        add_rule_file(search, &dir.join(".gitignore"), Some(anchor))?;
    }
    add_rule_file(search, &dir.join(".ignore"), Some(anchor))
}

/// Adds the patterns of one ignore file, relative to `root` if given.
///
/// Unreadable files leave the rules unapplied, matching the discovery
/// behavior of the standard ignore files.
fn add_rule_file(search: &mut Search, path: &Path, root: Option<&Path>) -> Result<()> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(()),
    };
    search
        .add_patterns_buffer(&bytes, path, root, Ignore::default())
        .with_context(|| format!("failed to load ignore file {}", path.display()))?;
    Ok(())
}

/// Converts a relative path to the slash-separated form used by ignore files.
fn to_slash(path: &Path) -> String {
    let mut result = String::new();
    for (index, component) in path.components().enumerate() {
        if index > 0 {
            result.push('/');
        }
        result.push_str(&component.as_os_str().to_string_lossy());
    }
    result
}

/// Returns the file configured as the global Git excludes file.
fn global_excludes_file() -> Option<PathBuf> {
    for config in global_config_files() {
        if let Some(excludes) = excludes_file_from_config(&config) {
            return Some(excludes);
        }
    }
    None
}

/// Returns the Git configuration files that can define the global excludes.
fn global_config_files() -> Vec<PathBuf> {
    if let Some(path) = env::var_os("GIT_CONFIG_GLOBAL") {
        return vec![PathBuf::from(path)];
    }
    let home = env::var_os("HOME").map(PathBuf::from);
    let mut files = Vec::new();
    if let Some(home) = &home {
        files.push(home.join(".gitconfig"));
    }
    let xdg = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|home| home.join(".config")));
    if let Some(xdg) = xdg {
        files.push(xdg.join("git").join("config"));
    }
    files
}

/// Reads `core.excludesFile` from one Git configuration file.
fn excludes_file_from_config(config: &Path) -> Option<PathBuf> {
    let text = fs::read_to_string(config).ok()?;
    let mut in_core = false;
    let mut excludes = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') {
            in_core = line.eq_ignore_ascii_case("[core]");
            continue;
        }
        if !in_core {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("excludesFile") {
            continue;
        }
        let value = value.trim().trim_matches('"');
        excludes = if value.is_empty() {
            None
        } else {
            let value = expand_home(value);
            Some(if value.is_absolute() {
                value
            } else {
                config
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(value)
            })
        };
    }
    excludes
}

/// Expands a leading `~/` in a configured path.
fn expand_home(value: &str) -> PathBuf {
    if let Some(rest) = value.strip_prefix("~/")
        && let Some(home) = env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(value)
}
