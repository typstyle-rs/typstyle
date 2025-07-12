// Borrowed from https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/source_kind.rs

use std::{borrow::Cow, path::Path};

use colored::Colorize;
use similar::{ChangeTag, TextDiff};

use crate::fs;

pub struct SourceDiff<'a> {
    pub original: &'a str,
    pub modified: &'a str,
    pub path: Option<&'a Path>,
}

impl std::fmt::Display for SourceDiff<'_> {
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

struct CodeDiff<'a> {
    diff: TextDiff<'a, 'a, 'a, str>,
    header: Option<(&'a str, &'a str)>,
    missing_newline_hint: bool,
}

impl<'a> CodeDiff<'a> {
    fn new(original: &'a str, modified: &'a str) -> Self {
        let diff = TextDiff::from_lines(original, modified);
        Self {
            diff,
            header: None,
            missing_newline_hint: true,
        }
    }

    fn header(&mut self, original: &'a str, modified: &'a str) {
        self.header = Some((original, modified));
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

        Ok(())
    }
}

// Borrowed from https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/text_helpers.rs
trait ShowNonprinting {
    fn show_nonprinting(&self) -> Cow<'_, str>;
}

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
