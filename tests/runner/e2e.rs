//! End-to-end trials over external Typst sources.

pub mod repos;

use anyhow::{Context, Result, bail};
use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, WrapMode};

/// The configuration profiles exercised by every end-to-end trial.
pub fn configs() -> [(&'static str, Config); 2] {
    [
        (
            "default",
            Config {
                reorder_import_items: true,
                ..Default::default()
            },
        ),
        (
            "reflow",
            Config {
                reorder_import_items: true,
                wrap_mode: WrapMode::Fill,
                ..Default::default()
            },
        ),
    ]
}

/// Builds a closure that formats a source in two passes, failing if the passes disagree.
///
/// Sources with syntax errors are rejected.
pub fn make_formatter(config: Config) -> impl Fn(Source) -> Result<String> {
    move |source| {
        let root = source.root();
        if root.diagnosis().errors {
            let (errors, _) = root.errors_and_warnings();
            bail!("the file has syntax errors: {errors:?}");
        }
        let t = Typstyle::new(config.clone());
        let first_pass = t.format_source(source).render().context("first pass")?;
        let second_pass = t.format_text(&first_pass).render().context("second pass")?;
        if first_pass != second_pass {
            bail!(
                "the formatting does not converge:\n{}",
                similar_asserts::SimpleDiff::from_str(
                    &first_pass,
                    &second_pass,
                    "first_pass",
                    "second_pass"
                )
            )
        }
        Ok(first_pass)
    }
}
