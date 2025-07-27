pub mod directive;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use typst_syntax::Source;
use typstyle_core::Config;

use self::directive::{parse_directives, process_includes};

pub fn test_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

pub fn fixtures_dir() -> PathBuf {
    test_dir().join("fixtures")
}

#[derive(Debug, Default)]
pub struct Options {
    pub config: Config,
    pub relax_convergence: usize,
    pub include_specs: Vec<String>,
}

pub fn read_source_with_options(path: &Path) -> Result<(Source, Options)> {
    let content = read_content(path)?;
    let options = parse_directives(&content)?;
    let final_content = process_includes(path, &content, &options)?;
    Ok((Source::detached(final_content), options))
}

pub fn read_source(path: &Path) -> Result<Source> {
    read_content(path).map(Source::detached)
}

pub fn read_content(path: &Path) -> Result<String> {
    let content = fs::read(path).context("Cannot read file")?;

    // Check that the file is valid UTF-8
    let content =
        String::from_utf8(content).context("The file's contents are not a valid UTF-8 string!")?;

    let content = remove_crlf(content);

    Ok(content)
}

fn remove_crlf(content: String) -> String {
    if cfg!(windows) {
        content.replace("\r\n", "\n")
    } else {
        content
    }
}
