//! Shared acquisition and validation of the pinned Typst documentation corpus.
//!
//! Both the docs end-to-end test and the `typst-docs` benchmark format the same
//! upstream sources. The checkout is cached by revision under `.cache/docs`, so
//! bumping the pin cannot silently reuse an older checkout.

use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

use crate::{fixtures::test_dir, git_checkout};

/// The Git repository from which the docs corpus is fetched.
pub const TYPST_DOCS_REPO: &str = "https://github.com/typst/typst.git";
/// The Typst revision whose docs are used. Keep in sync with the workspace's
/// `typst-syntax` version.
pub const TYPST_DOCS_REV: &str = "v0.15.1";

/// A validated checkout of the pinned Typst documentation corpus.
pub struct DocsCorpus {
    /// The `docs` directory inside the checkout.
    docs: PathBuf,
}

impl DocsCorpus {
    /// Returns the corpus' `docs` directory.
    pub fn docs_dir(&self) -> &Path {
        &self.docs
    }

    /// Returns every `.typ` file below [`Self::docs_dir`], sorted by path.
    pub fn typ_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        collect_typ_files(&self.docs, &mut files)?;
        files.sort();
        Ok(files)
    }
}

/// Returns the validated docs corpus, fetching the pinned revision on first use.
///
/// The cache is keyed by [`TYPST_DOCS_REV`] under `.cache/docs`. Access is
/// serialized with a per-revision lock file, and a cold checkout is prepared in
/// a temporary staging directory before being renamed into place, so concurrent
/// runs never observe a partial corpus or discard a valid checkout.
pub fn load() -> Result<DocsCorpus> {
    load_from(&cache_root(), TYPST_DOCS_REPO, TYPST_DOCS_REV)
}

/// Loads or acquires a pinned docs corpus below `root`.
fn load_from(root: &Path, repo: &str, rev: &str) -> Result<DocsCorpus> {
    fs::create_dir_all(root)
        .with_context(|| format!("cannot create the docs corpus cache `{}`", root.display()))?;
    let key = revision_key(rev);
    let target = root.join(format!("typst-{key}"));
    let _lock = acquire_lock(root, &key)?;

    if let Some(corpus) = reuse(&target, rev)? {
        return Ok(corpus);
    }

    let staging = tempfile::Builder::new()
        .prefix(".typst-docs-")
        .tempdir_in(root)
        .with_context(|| format!("cannot create a staging directory in `{}`", root.display()))?;
    prepare(staging.path(), repo, rev)?;
    remove_stale(&target)?;
    fs::rename(staging.path(), &target)
        .with_context(|| format!("cannot publish the docs corpus at `{}`", target.display()))?;
    let _ = staging.keep();

    Ok(DocsCorpus {
        docs: target.join("docs"),
    })
}

/// Returns the shared cache directory for fetched corpora.
fn cache_root() -> PathBuf {
    test_dir().join(".cache").join("docs")
}

/// Turns a revision into a portable cache key.
fn revision_key(rev: &str) -> String {
    rev.chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => ch,
            _ => '_',
        })
        .collect()
}

/// Acquires an exclusive lock file for a revision's cache entry.
fn acquire_lock(root: &Path, key: &str) -> Result<File> {
    let path = root.join(format!(".typst-{key}.lock"));
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("cannot open docs corpus lock `{}`", path.display()))?;
    file.lock()
        .with_context(|| format!("cannot lock docs corpus cache `{}`", path.display()))?;
    Ok(file)
}

/// Validates and returns an existing keyed checkout.
fn reuse(target: &Path, rev: &str) -> Result<Option<DocsCorpus>> {
    if !target.is_dir() {
        return Ok(None);
    }
    let docs = target.join("docs");
    if !docs.join("lib.typ").is_file() || !git_checkout::is_at_revision(target, Some(rev))? {
        return Ok(None);
    }
    Ok(Some(DocsCorpus { docs }))
}

/// Clones and prepares the corpus in an empty staging directory.
fn prepare(dir: &Path, repo: &str, rev: &str) -> Result<()> {
    run_git(
        Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                rev,
                "--filter=blob:none",
                "--sparse",
                repo,
            ])
            .arg(dir),
        &format!("git clone {repo} at {rev}"),
    )?;

    run_git(
        Command::new("git")
            .args(["sparse-checkout", "set", "docs"])
            .current_dir(dir),
        "git sparse-checkout set docs",
    )?;

    if reuse(dir, rev)?.is_none() {
        bail!("the fetched Typst docs are incomplete or not at `{rev}`");
    }
    Ok(())
}

/// Runs a `git` command, capturing output and reporting stderr on failure.
fn run_git(cmd: &mut Command, description: &str) -> Result<()> {
    let output = cmd
        .output()
        .with_context(|| format!("failed to run `{description}`"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        if stderr.is_empty() {
            bail!("`{description}` failed with {}", output.status);
        }
        bail!("`{description}` failed with {}: {stderr}", output.status);
    }
    Ok(())
}

/// Removes a stale cache entry at `path`, if any.
fn remove_stale(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(err).with_context(|| {
                format!("cannot inspect stale docs checkout at `{}`", path.display())
            });
        }
    };
    let result = if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    result.with_context(|| format!("cannot remove stale docs checkout `{}`", path.display()))
}

/// Collects all `.typ` files below a directory.
fn collect_typ_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("cannot read `{}`", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_typ_files(&path, files)?;
        } else if path.extension() == Some("typ".as_ref()) {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Barrier},
        thread,
    };

    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("spawn git");
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn file_url(path: &Path) -> String {
        let canonical = fs::canonicalize(path).expect("canonicalize origin");
        let text = canonical.to_string_lossy();
        let stripped = text.strip_prefix(r"\\?\").unwrap_or(&text);
        let forward = stripped.replace('\\', "/");
        if forward.starts_with('/') {
            format!("file://{forward}")
        } else {
            format!("file:///{forward}")
        }
    }

    fn make_origin_repo(dir: &Path, rev: &str) -> String {
        git(dir, &["init", "-b", "main"]);
        git(dir, &["config", "user.name", "Typstyle Test"]);
        git(dir, &["config", "user.email", "test@example.com"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        git(dir, &["config", "tag.gpgsign", "false"]);
        git(dir, &["config", "uploadpack.allowFilter", "true"]);

        fs::create_dir_all(dir.join("docs/sub")).expect("create docs dir");
        fs::write(dir.join("docs/lib.typ"), "= Docs\n").expect("write lib.typ");
        fs::write(dir.join("docs/sub/page.typ"), "Hello world.\n").expect("write page.typ");
        fs::write(dir.join("other.typ"), "ignored\n").expect("write other.typ");

        git(dir, &["add", "."]);
        git(dir, &["commit", "-m", "initial"]);
        git(dir, &["tag", rev]);
        file_url(dir)
    }

    #[test]
    fn load_replaces_stale_directory_and_file() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let origin = temp.path().join("origin");
        let cache = temp.path().join("cache");
        fs::create_dir_all(&origin).expect("create origin");
        let url = make_origin_repo(&origin, "v0.15.1");
        let target = cache.join("typst-v0.15.1");

        // Stale regular file at the target path is replaced cleanly.
        fs::create_dir_all(&cache).expect("create cache");
        fs::write(&target, "not a checkout").expect("write stale file");
        let corpus = load_from(&cache, &url, "v0.15.1").expect("load over stale file");
        assert_eq!(corpus.typ_files().expect("list files").len(), 2);

        // Stale directory (missing lib.typ) is also replaced cleanly.
        fs::remove_file(target.join("docs/lib.typ")).expect("corrupt cached checkout");
        let corpus = load_from(&cache, &url, "v0.15.1").expect("load over stale dir");
        assert_eq!(corpus.typ_files().expect("list files").len(), 2);
    }

    #[test]
    fn concurrent_loads_never_discard_published_checkout() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let origin = temp.path().join("origin");
        let cache = temp.path().join("cache");
        fs::create_dir_all(&origin).expect("create origin");
        let url = make_origin_repo(&origin, "v0.15.1");

        let workers = 6;
        let barrier = Arc::new(Barrier::new(workers));
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let cache = cache.clone();
                let url = url.clone();
                thread::spawn(move || {
                    barrier.wait();
                    let corpus = load_from(&cache, &url, "v0.15.1").expect("concurrent load");
                    let files = corpus.typ_files().expect("enumerate typ files");
                    assert_eq!(files.len(), 2);
                    for file in files {
                        let content = fs::read_to_string(&file).expect("read corpus file");
                        assert!(!content.is_empty());
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("worker should succeed");
        }
    }

    #[test]
    fn clone_failure_includes_git_stderr() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let origin = temp.path().join("origin");
        let cache = temp.path().join("cache");
        fs::create_dir_all(&origin).expect("create origin");
        let url = make_origin_repo(&origin, "v0.15.1");

        let err = load_from(&cache, &url, "v999.0.0")
            .err()
            .expect("missing tag should fail");
        let message = format!("{err:#}");
        assert!(
            message.contains("v999.0.0"),
            "error should mention the requested revision: {message}"
        );
        assert!(
            message.contains("Remote branch") || message.contains("not found"),
            "error should include git stderr diagnostics: {message}"
        );
    }
}
