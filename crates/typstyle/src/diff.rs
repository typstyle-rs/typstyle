//! Rendering of unified diffs of formatting changes.

// Borrowed from https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/source_kind.rs

use std::{borrow::Cow, path::Path};

use colored::Colorize;
use similar::{ChangeTag, TextDiff};

use crate::fs;

/// A unified diff of the original and formatted content of one source.
pub struct SourceDiff<'a> {
    /// The source text before formatting.
    pub original: &'a str,
    /// The source text after formatting.
    pub modified: &'a str,
    /// The path of the formatted file, if known; shown in the diff header.
    pub path: Option<&'a Path>,
}

impl std::fmt::Display for SourceDiff<'_> {
    /// Writes the unified diff, headed by the relative path if one is given.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut diff = CodeDiff::new(self.original, self.modified);

        let relative_path = self.path.map(fs::relativize_path);
        if let Some(relative_path) = &relative_path {
            diff.header(relative_path, relative_path);
        }

        writeln!(f, "{diff}")?;

        Ok(())
    }
}

/// The unified diff body between two texts.
struct CodeDiff<'a> {
    /// The line-based diff between the original and modified texts.
    diff: TextDiff<'a, 'a, str>,
    /// The file names printed in the `---`/`+++` header lines, if any.
    header: Option<(&'a str, &'a str)>,
    /// Whether to print the `\ No newline at end of file` hint.
    missing_newline_hint: bool,
}

impl<'a> CodeDiff<'a> {
    /// Creates a diff between the two texts, without a header.
    fn new(original: &'a str, modified: &'a str) -> Self {
        let diff = TextDiff::from_lines(original, modified);
        Self {
            diff,
            header: None,
            missing_newline_hint: true,
        }
    }

    /// Sets the file names shown in the `---`/`+++` header lines.
    fn header(&mut self, original: &'a str, modified: &'a str) {
        self.header = Some((original, modified));
    }
}

impl std::fmt::Display for CodeDiff<'_> {
    /// Writes the header lines and the unified diff hunks.
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
/// Replaces non-printing characters in a string with visible symbols.
trait ShowNonprinting {
    /// Returns the string with its non-printing characters escaped.
    fn show_nonprinting(&self) -> Cow<'_, str>;
}

/// Implements [`ShowNonprinting`] for `str`, escaping the given characters.
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
