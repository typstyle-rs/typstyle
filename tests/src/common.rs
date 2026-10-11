//! Shared infrastructure for reading test fixtures and their embedded directives.

pub mod directive;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use typst_syntax::Source;
use typstyle_core::Config;

use self::directive::{parse_directives, process_includes};

/// Returns the root directory of the tests crate.
pub fn test_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Returns the directory containing the test fixtures.
pub fn fixtures_dir() -> PathBuf {
    test_dir().join("fixtures")
}

/// Options that control how a fixture is formatted, parsed from its directives.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Options {
    /// The style configuration to format the fixture with.
    pub config: Config,
    /// The number of unstable formatting iterations tolerated before the convergence
    /// check fails.
    pub relax_convergence: usize,
    /// Whether to skip the convergence and consistency checks for this fixture.
    pub skip_consistency: bool,
    /// The include specifications to resolve, in order of appearance.
    pub include_specs: Vec<String>,
}

/// Reads a fixture file, parses its directives, and resolves its includes.
///
/// Returns the combined content as a detached [`Source`] together with the parsed [`Options`].
pub fn read_source_with_options(path: &Path) -> Result<(Source, Options)> {
    let content = read_content(path)?;
    let options = parse_directives(&content)?;
    let final_content = process_includes(path, &content, &options)?;
    Ok((Source::detached(final_content), options))
}

/// Reads a fixture file into a detached [`Source`] without parsing directives.
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
