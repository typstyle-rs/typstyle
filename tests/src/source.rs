//! Reading source files used by the test runner and benchmarks.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use typst_syntax::Source;

/// Reads a file into a detached [`Source`] without parsing fixture directives.
pub fn read_source(path: &Path) -> Result<Source> {
    read_content(path).map(Source::detached)
}

/// Reads a file as UTF-8 and normalizes its CRLF line endings to LF.
pub fn read_content(path: &Path) -> Result<String> {
    let content = fs::read(path).context("Cannot read file")?;

    // Check that the file is valid UTF-8
    let content =
        String::from_utf8(content).context("The file's contents are not a valid UTF-8 string!")?;

    let content = remove_crlf(content);

    Ok(content)
}

/// Replaces CRLF line endings with LF when running on Windows.
fn remove_crlf(content: String) -> String {
    if cfg!(windows) {
        content.replace("\r\n", "\n")
    } else {
        content
    }
}
