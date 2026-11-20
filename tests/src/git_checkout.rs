//! Reuse-or-clone handling for the Git checkouts used by end-to-end tests.

use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};

/// Returns whether `repo` is a Git checkout at the expected revision.
///
/// With no expected revision, any usable checkout is accepted, so a cached
/// directory is reused without knowing which commit it is at.
pub fn is_at_revision(repo: &Path, rev: Option<&str>) -> Result<bool> {
    if !repo.is_dir() || !is_work_tree(repo)? {
        return Ok(false);
    }

    let Some(rev) = rev else {
        return Ok(true);
    };

    let Some(head) = git_stdout(repo, &["rev-parse", "HEAD"])? else {
        return Ok(false);
    };
    let expected = format!("{rev}^{{commit}}");
    let Some(expected) = git_stdout(repo, &["rev-parse", &expected])? else {
        return Ok(false);
    };
    Ok(head.trim() == expected.trim())
}

/// Returns whether `repo` itself is the root of a Git work tree.
///
/// Git resolves repositories from parent directories, so a plain directory
/// nested in another checkout would otherwise appear to be a valid checkout.
fn is_work_tree(repo: &Path) -> Result<bool> {
    let Some(toplevel) = git_stdout(repo, &["rev-parse", "--show-toplevel"])? else {
        return Ok(false);
    };
    let Ok(toplevel) = fs::canonicalize(toplevel.trim()) else {
        return Ok(false);
    };
    Ok(toplevel == fs::canonicalize(repo)?)
}

/// Runs `git` in `repo`, returning `None` when it exits unsuccessfully.
fn git_stdout(repo: &Path, args: &[&str]) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| {
            format!(
                "failed to run `git {}` in `{}`",
                args.join(" "),
                repo.display()
            )
        })?;
    if !output.status.success() {
        return Ok(None);
    }
    String::from_utf8(output.stdout)
        .map(Some)
        .context("`git` produced non-UTF-8 output")
}
