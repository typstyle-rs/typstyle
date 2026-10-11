// Borrowed from https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/source_kind.rs
// Copied from crates/typstyle/src/diff.rs

//! Renders source differences as colored unified diffs for consistency reports.

use std::borrow::Cow;

use colored::Colorize;
use similar::{ChangeTag, TextDiff};

/// A unified diff between two source texts, rendered in color on display.
pub struct CodeDiff<'a> {
    /// The computed line-based diff.
    diff: TextDiff<'a, 'a, str>,
    /// The optional file names shown as `---`/`+++` header lines.
    header: Option<(&'a str, &'a str)>,
    /// Whether to hint at missing trailing newlines.
    missing_newline_hint: bool,
}

impl<'a> CodeDiff<'a> {
    /// Creates a diff between the original and modified texts, compared line by line.
    pub fn new(original: &'a str, modified: &'a str) -> Self {
        let diff = TextDiff::from_lines(original, modified);
        Self {
            diff,
            header: None,
            missing_newline_hint: true,
        }
    }
}

impl std::fmt::Display for CodeDiff<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((original, modified)) = self.header {
            writeln!(f, "--- {}", original.show_nonprinting().red())?;
            writeln!(f, "+++ {}", modified.show_nonprinting().green())?;
        }

        let mut unified = self.diff.unified_diff();
        unified.missing_newline_hint(self.missing_newline_hint);

        // Individual hunks (section of changes)
        for hunk in unified.iter_hunks() {
            writeln!(f, "{}", hunk.header())?;

            // individual lines
            for change in hunk.iter_changes() {
                let value = change.value().show_nonprinting();
                match change.tag() {
                    ChangeTag::Equal => write!(f, " {value}")?,
                    ChangeTag::Delete => write!(f, "{}{}", "-".red(), value.red())?,
                    ChangeTag::Insert => write!(f, "{}{}", "+".green(), value.green())?,
                }

                if !self.diff.newline_terminated() {
                    writeln!(f)?;
                } else if change.missing_newline() {
                    if self.missing_newline_hint {
                        writeln!(f, "{}", "\n\\ No newline at end of file".red())?;
                    } else {
                        writeln!(f)?;
                    }
                }
            }
        }

        Ok(())
    }
}

// Borrowed from https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/text_helpers.rs
/// Renders control characters visibly so diffs stay readable.
trait ShowNonprinting {
    /// Returns the text with control characters replaced by visible symbols.
    fn show_nonprinting(&self) -> Cow<'_, str>;
}

/// Implements [`ShowNonprinting`] for `str`, replacing the given control characters.
macro_rules! impl_show_nonprinting {
    ($(($from:expr, $to:expr)),+) => {
        impl ShowNonprinting for str {
            fn show_nonprinting(&self) -> Cow<'_, str> {
                if self.find(&[$($from),*][..]).is_some() {
                    Cow::Owned(
                        self.$(replace($from, $to)).*
                    )
                } else {
                    Cow::Borrowed(self)
                }
            }
        }
    };
}

impl_show_nonprinting!(('\x07', "␇"), ('\x08', "␈"), ('\x1b', "␛"), ('\x7f', "␡"));
