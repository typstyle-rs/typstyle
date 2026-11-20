//! End-to-end tests over the official Typst documentation sources.
//!
//! The docs are a large, prose-heavy corpus maintained by the Typst project. They
//! cannot be compiled with a stock Typst library because their build injects an
//! `stdx` module from `typst-docs`. This group therefore checks the properties
//! that do not need compilation: every file parses, formats, and reaches a fixed
//! point.

use std::{fmt::Write, path::Path};

use anyhow::{Context, Result, bail};
use libtest_mimic::{Failed, Trial};
use typst_syntax::Source;
use typstyle_tests::{docs_corpus, source::read_source};

use crate::e2e::{configs, make_formatter};

/// Creates the single trial that formats the whole docs corpus.
pub fn collect_tests() -> Vec<Trial> {
    vec![
        Trial::test("typst-docs", || {
            run().map_err(|err| Failed::from(format!("{err:#}")))
        })
        .with_kind("e2e"),
    ]
}

/// Formats every docs file with every configuration, collecting all failures.
fn run() -> Result<()> {
    let corpus = docs_corpus::load()?;
    let docs = corpus.docs_dir();
    let files = corpus.typ_files()?;
    if files.is_empty() {
        bail!("no `.typ` files found in `{}`", docs.display());
    }

    let sources: Vec<(String, Source)> = files
        .iter()
        .map(|path| {
            let rel = relative_path(path, docs);
            read_source(path)
                .with_context(|| format!("cannot read `{rel}`"))
                .map(|source| (rel, source))
        })
        .collect::<Result<_>>()?;

    let mut errors = Vec::new();
    for (name, config) in configs() {
        let format = make_formatter(config);
        for (path, source) in &sources {
            if let Err(err) = format(source.clone()) {
                errors.push(format!("`{path}` with the `{name}` config: {err:#}"));
            }
        }
    }

    if errors.is_empty() {
        return Ok(());
    }
    let mut report = String::from("errors occurred in typst docs e2e:");
    for (i, err) in errors.iter().enumerate() {
        let err = err.replace('\n', "\n    ");
        let _ = write!(report, "\n{:4}: {err}", i + 1);
    }
    bail!("{report}")
}

/// Renders a path relative to the docs root with forward slashes.
fn relative_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
