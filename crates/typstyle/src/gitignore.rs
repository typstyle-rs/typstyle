//! Ignore rules for directory discovery.
//!
//! This module owns `.ignore`, `.gitignore`, and Git's local excludes,
//! including repository boundaries and the lifetime of per-directory rules.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use bstr::BStr;
use gix_ignore::{Search, glob::pattern::Case, search::Ignore};

/// The active ignore rules while walking one root.
pub(crate) struct IgnoreWalk {
    respect: bool,
    /// `.ignore` rules, which stay active across repository boundaries.
    ignore: Search,
    /// The directory the `.ignore` paths are relative to.
    ignore_anchor: PathBuf,
    /// Git-specific rules.
    git: GitIgnore,
    /// The directories that were entered, innermost last.
    frames: Vec<IgnoreFrame>,
}

/// Saved state of a directory, so its `.ignore` rules can be dropped when left.
struct IgnoreFrame {
    depth: usize,
    ignore_len: usize,
}

impl IgnoreWalk {
    /// Collects the rules that apply to a walk rooted at `root`.
    pub(crate) fn new(root: &Path, respect: bool) -> Result<Self> {
        let mut walk = Self {
            respect,
            ignore: Search::default(),
            ignore_anchor: root.to_path_buf(),
            git: GitIgnore::new(root, respect)?,
            frames: Vec::new(),
        };
        if !respect {
            return Ok(walk);
        }

        let ancestors = ancestor_ignore_files(root);
        if let Some(anchor) = ancestors.first().and_then(|file| file.parent()) {
            walk.ignore_anchor = anchor.to_path_buf();
        }
        for file in &ancestors {
            add_rule_file(&mut walk.ignore, file, Some(walk.ignore_anchor.as_path()))?;
        }
        Ok(walk)
    }

    /// Loads the rules of `dir` for the entries below it.
    pub(crate) fn enter(&mut self, dir: &Path, depth: usize) -> Result<()> {
        if !self.respect {
            return Ok(());
        }
        let ignore_len = rule_count(&self.ignore);
        add_rule_file(
            &mut self.ignore,
            &dir.join(".ignore"),
            Some(self.ignore_anchor.as_path()),
        )?;
        self.git.enter(dir, depth)?;
        self.frames.push(IgnoreFrame { depth, ignore_len });
        Ok(())
    }

    /// Drops the rules of directories the walk has left.
    pub(crate) fn leave(&mut self, depth: usize) {
        if !self.respect {
            return;
        }
        self.git.leave(depth);
        while self.frames.last().is_some_and(|frame| frame.depth >= depth) {
            let frame = self.frames.pop().expect("frame exists");
            truncate_rules(&mut self.ignore, frame.ignore_len);
        }
    }

    /// Returns whether the last matching rule excludes `path`.
    pub(crate) fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        if !self.respect {
            return false;
        }
        let mut ignored = self.git.is_ignored(path, is_dir);
        // `.ignore` takes precedence over every Git source, regardless of the
        // directory depth of the Git rule it overrides.
        if let Some(matched) = search_match(&self.ignore, path, &self.ignore_anchor, is_dir) {
            ignored = matched;
        }
        ignored
    }
}

/// The ignore rules that belong to one repository.
struct RepoRules {
    /// The root of the worktree the rules apply to.
    root: PathBuf,
    /// The static rules and `.gitignore` files in ascending precedence.
    search: Search,
}

impl RepoRules {
    /// Collects the rules that are not tied to one directory of the repository.
    fn new(git_root: &Path) -> Result<Self> {
        let git_dir = resolve_git_dir(git_root);
        let common_dir = resolve_common_dir(&git_dir);
        let mut search = Search::default();
        add_rule_file(&mut search, &common_dir.join("info").join("exclude"), None)?;
        Ok(Self {
            root: git_root.to_path_buf(),
            search,
        })
    }

    /// Loads the `.gitignore` rules of `dir` for the entries below it.
    fn add_gitignore(&mut self, dir: &Path) -> Result<()> {
        add_rule_file(
            &mut self.search,
            &dir.join(".gitignore"),
            Some(self.root.as_path()),
        )
    }

    /// Returns whether the last matching rule excludes `path`.
    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        search_match(&self.search, path, &self.root, is_dir).unwrap_or(false)
    }
}

/// Saved state of a directory, so its Git rules can be dropped when the walk leaves it.
struct GitFrame {
    depth: usize,
    repo_len: Option<usize>,
    entered_repo: bool,
}

/// The active Git rules while walking one root.
struct GitIgnore {
    respect: bool,
    /// The enclosing repositories, innermost last.
    repos: Vec<RepoRules>,
    /// The directories that were entered, innermost last.
    frames: Vec<GitFrame>,
}

impl GitIgnore {
    /// Collects the Git rules that apply to a walk rooted at `root`.
    fn new(root: &Path, respect: bool) -> Result<Self> {
        let mut ignore = Self {
            respect,
            repos: Vec::new(),
            frames: Vec::new(),
        };
        if !respect {
            return Ok(ignore);
        }
        if let Some(repo_root) = find_repo_root(root) {
            let mut repo = RepoRules::new(&repo_root)?;
            if root != repo_root {
                for dir in ancestor_dirs(&repo_root, root) {
                    repo.add_gitignore(&dir)?;
                }
            }
            ignore.repos.push(repo);
        }
        Ok(ignore)
    }

    /// Loads the Git rules of `dir` for the entries below it.
    fn enter(&mut self, dir: &Path, depth: usize) -> Result<()> {
        if !self.respect {
            return Ok(());
        }
        let mut entered_repo = false;
        if (dir.join(".git").exists() || dir.join(".jj").exists())
            && self
                .repos
                .last()
                .is_none_or(|repo| repo.root.as_path() != dir)
        {
            let repo = RepoRules::new(dir)?;
            self.repos.push(repo);
            entered_repo = true;
        }

        let repo_len = self.repos.last().map(|repo| rule_count(&repo.search));
        if let Some(repo) = self.repos.last_mut() {
            repo.add_gitignore(dir)?;
        }
        self.frames.push(GitFrame {
            depth,
            repo_len,
            entered_repo,
        });
        Ok(())
    }

    /// Drops the Git rules of directories the walk has left.
    fn leave(&mut self, depth: usize) {
        if !self.respect {
            return;
        }
        while self.frames.last().is_some_and(|frame| frame.depth >= depth) {
            let frame = self.frames.pop().expect("frame exists");
            if frame.entered_repo {
                self.repos.pop();
            } else if let (Some(len), Some(repo)) = (frame.repo_len, self.repos.last_mut()) {
                truncate_rules(&mut repo.search, len);
            }
        }
    }

    /// Returns whether the last matching Git rule excludes `path`.
    fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        self.respect
            && self
                .repos
                .last()
                .is_some_and(|repo| repo.is_ignored(path, is_dir))
    }
}

/// Returns the `.ignore` files above `root`, outermost first.
fn ancestor_ignore_files(root: &Path) -> Vec<PathBuf> {
    let mut files = root
        .ancestors()
        .skip(1)
        .map(|dir| dir.join(".ignore"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    files.reverse();
    files
}

/// Returns the number of rule lists active in `search`.
fn rule_count(search: &Search) -> usize {
    search.patterns.len()
}

/// Drops the rule lists of `search` above `len`.
fn truncate_rules(search: &mut Search, len: usize) {
    search.patterns.truncate(len);
}

/// Returns the match result of `search` for `path` relative to `base`.
fn search_match(search: &Search, path: &Path, base: &Path, is_dir: bool) -> Option<bool> {
    let relative = relative_slash_path(path, base)?;
    search
        .pattern_matching_relative_path(
            BStr::new(relative.as_bytes()),
            Some(is_dir),
            Case::Sensitive,
        )
        .map(|matched| !matched.pattern.is_negative())
}

/// Returns `path` relative to `base`, using slashes, or `None` when empty.
fn relative_slash_path(path: &Path, base: &Path) -> Option<String> {
    let relative = path.strip_prefix(base).ok()?;
    let relative = to_slash(relative);
    (!relative.is_empty()).then_some(relative)
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
    validate_patterns(&bytes)
        .with_context(|| format!("failed to parse ignore file {}", path.display()))?;
    search
        .add_patterns_buffer(&bytes, path, root, Ignore::default())
        .with_context(|| format!("failed to load ignore file {}", path.display()))?;
    Ok(())
}

/// Rejects patterns with a dangling escape, which the glob parser cannot match.
fn validate_patterns(bytes: &[u8]) -> Result<()> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    for line in bytes.split(|byte| *byte == b'\n') {
        let line = line.strip_suffix(b"\r".as_slice()).unwrap_or(line);
        if line.first() == Some(&b'#') {
            continue;
        }
        if has_trailing_escape(line) {
            bail!("invalid ignore pattern {:?}", String::from_utf8_lossy(line));
        }
    }
    Ok(())
}

/// Returns whether the pattern ends with an unescaped backslash.
fn has_trailing_escape(line: &[u8]) -> bool {
    line.iter().rev().take_while(|byte| **byte == b'\\').count() % 2 == 1
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

/// Returns the nearest ancestor that contains a `.git` or `.jj` entry.
fn find_repo_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|dir| dir.join(".git").exists() || dir.join(".jj").exists())
        .map(Path::to_path_buf)
}

/// Returns the directories from `repo_root` to the parent of `root`, outermost first.
fn ancestor_dirs(repo_root: &Path, root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut current = root.parent();
    while let Some(dir) = current {
        if !dir.starts_with(repo_root) {
            break;
        }
        dirs.push(dir.to_path_buf());
        if dir == repo_root {
            break;
        }
        current = dir.parent();
    }
    dirs.reverse();
    dirs
}

/// Returns the Git directory of `git_root`, following a `.git` pointer file.
fn resolve_git_dir(git_root: &Path) -> PathBuf {
    let dot_git = git_root.join(".git");
    if dot_git.is_dir() {
        return dot_git;
    }
    let Ok(text) = fs::read_to_string(&dot_git) else {
        return dot_git;
    };
    let Some(path) = text
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("gitdir:"))
    else {
        return dot_git;
    };
    let path = PathBuf::from(path.trim());
    if path.is_absolute() {
        path
    } else {
        git_root.join(path)
    }
}

/// Returns the common Git directory, which holds `info/exclude`.
fn resolve_common_dir(git_dir: &Path) -> PathBuf {
    let Ok(text) = fs::read_to_string(git_dir.join("commondir")) else {
        return git_dir.to_path_buf();
    };
    let path = PathBuf::from(text.trim());
    if path.is_absolute() {
        path
    } else {
        git_dir.join(path)
    }
}
