//! Fixture loading and the directives embedded at the top of fixture files.

pub mod directive;

use std::path::{Path, PathBuf};

use anyhow::Result;
use typst_syntax::Source;
use typstyle_core::Config;

use self::directive::{parse_directives, process_includes};
use crate::source::read_content;

/// Returns the root directory of the tests crate.
pub fn test_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Returns the root directory containing the test fixtures.
pub fn root() -> PathBuf {
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
