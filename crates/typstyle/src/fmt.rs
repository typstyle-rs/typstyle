/// This module provides functionality to format Typst files either in-place or by checking
/// their formatting via standard input/output.
///
/// Adapted from: https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/fs.rs
mod line_endings;

use std::{
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use itertools::Itertools;
use line_endings::apply_crlf_preserve;
use log::{debug, error, info, warn};
use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, format_ast};
use walkdir::{DirEntry, WalkDir};

use crate::{
    ExitStatus,
    cli::{CliArguments, DebugArgs, LineEndingMode, StyleArgs},
    diff::SourceDiff,
    fs,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum FormatMode {
    /// Write the formatted contents back to the file.
    Write,
    /// Check if the file is formatted, but do not write the formatted contents back.
    Check,
    /// Show unified diff of what formatting changes would be made.
    Diff,
}

impl FormatMode {
    pub(crate) fn from_cli(cli: &CliArguments) -> Self {
        if cli.check {
            FormatMode::Check
        } else if cli.diff {
            FormatMode::Diff
        } else {
            FormatMode::Write
        }
    }
}

impl StyleArgs {
    pub fn to_config(&self) -> Config {
        use typstyle_core::WrapMode;

        use crate::cli::WrapTextMode;

        Config {
            max_width: self.line_width,
            tab_spaces: self.indent_width,
            reorder_import_items: !self.no_reorder_import_items,
            wrap_mode: match self.wrap_text {
                WrapTextMode::None => WrapMode::None,
                WrapTextMode::Fill => WrapMode::Fill,
                WrapTextMode::Sentence => WrapMode::Sentence,
                WrapTextMode::FillSentence => WrapMode::FillSentence,
            },
            ..Default::default()
        }
    }
}

pub fn format_stdin(args: &CliArguments) -> Result<ExitStatus> {
    let typstyle = Typstyle::new(args.style.to_config());

    format_one(None, &typstyle, args).map(|res| match res {
        FormatResult::Formatted(_) if args.check || args.diff => ExitStatus::Failure,
        _ => ExitStatus::Success,
    })
}

pub fn format(args: &CliArguments) -> Result<ExitStatus> {
    #[derive(Default)]
    struct Summary {
        format_count: usize,
        unchanged_count: usize,
        error_count: usize,
    }
    let mut summary = Summary::default();

    let mode = FormatMode::from_cli(args);
    let paths = resolve_typst_files(&args.input);
    if paths.is_empty() {
        warn!("No Typst files found under the given path(s).");
        return Ok(ExitStatus::Success);
    }

    let typstyle = Typstyle::new(args.style.to_config());

    let start_time = Instant::now();
    for file in paths {
        let res = format_one(Some(&file), &typstyle, args).unwrap_or_else(|e| {
            error!("{e}");
            summary.error_count += 1;
            FormatResult::Erroneous
        });

        // Check if the content is already well-formatted (unchanged)
        match res {
            FormatResult::Formatted(_) => summary.format_count += 1,
            _ => summary.unchanged_count += 1,
        }
    }
    let duration = start_time.elapsed();

    fn num_files(num: usize) -> String {
        if num > 1 {
            format!("{num} files")
        } else {
            format!("{num} file")
        }
    }

    match mode {
        FormatMode::Write => debug!(
            "Successfully formatted {} ({} unchanged) in {:?}",
            num_files(summary.format_count),
            summary.unchanged_count,
            duration
        ),
        FormatMode::Check => debug!(
            "{} would be reformatted ({} already formatted), checked in {:?}",
            num_files(summary.format_count),
            summary.unchanged_count,
            duration
        ),
        FormatMode::Diff => debug!(
            "{} would be reformatted ({} already formatted), checked with diff in {:?}",
            num_files(summary.format_count),
            summary.unchanged_count,
            duration
        ),
    }
    if summary.error_count > 0 {
        // Syntax errors are not counted here.
        bail!(
            "failed to format {} due to IO error",
            num_files(summary.error_count)
        );
    }

    Ok(match mode {
        FormatMode::Check | FormatMode::Diff if summary.format_count > 0 => ExitStatus::Failure,
        _ => ExitStatus::Success,
    })
}

/// Formats a single `.typ` file or input from stdin.
///
/// This function formats the file provided as an argument, or reads from stdin if no file is given.
/// If in-place formatting is requested, it overwrites the file with the formatted content.
///
/// # Parameters
/// - `input`: An optional path to a `.typ` file to be formatted. If `None`, input is read from stdin.
/// - `args`: CLI arguments.
///
/// # Returns
/// - `Ok(FormatStatus::Changed)` if the file was reformatted.
/// - `Ok(FormatStatus::Unchanged)` if the file was unchanged or contained errors.
/// - `Err` if reading from or writing to the file fails.
fn format_one(
    input: Option<&Path>,
    typstyle: &Typstyle,
    args: &CliArguments,
) -> Result<FormatResult> {
    let use_stdout = !args.inplace && !args.check && !args.diff;
    let unformatted = get_input(input)?;

    let res = format_debug(&unformatted, typstyle, &args.debug)
        .apply_line_ending(&unformatted, args.style.line_ending);
    match &res {
        FormatResult::Formatted(res) => {
            if args.inplace {
                // We have already validated that the input is Some.
                write_back(input.unwrap(), res)?;
            } else if args.check {
                if let Some(path) = input {
                    info!("Would reformat: {}", fs::relativize_path(path));
                } else {
                    // For stdin, we don't output anything in check mode
                    // just rely on the exit code
                }
            } else if args.diff {
                print_unified_diff(&unformatted, res, input);
            } else {
                print!("{res}");
            }
        }
        FormatResult::Unchanged => {
            if use_stdout {
                print!("{unformatted}");
            }
        }
        FormatResult::Erroneous => {
            if use_stdout {
                print!("{unformatted}"); // still prints the original content to enable piping
            }
            if let Some(path) = input {
                warn!(
                    "Failed to parse {}. The source is erroneous.",
                    fs::relativize_path(path)
                );
            } else {
                warn!("Failed to parse stdin. The source is erroneous.");
            }
        }
    }
    Ok(res)
}

enum FormatResult {
    Formatted(String),
    Unchanged,
    Erroneous,
}

impl FormatResult {
    fn apply_line_ending(self, original: &str, mode: LineEndingMode) -> Self {
        let Self::Formatted(formatted) = self else {
            return self;
        };
        let formatted = match mode {
            LineEndingMode::Lf => formatted,
            LineEndingMode::CrlfPreserve => apply_crlf_preserve(original, formatted),
        };

        if formatted == original {
            Self::Unchanged
        } else {
            Self::Formatted(formatted)
        }
    }
}

fn format_debug(content: &str, typstyle: &Typstyle, args: &DebugArgs) -> FormatResult {
    let source = Source::detached(content);
    let root = source.root();
    if args.ast {
        println!("{}", format_ast(root));
    }

    let start_time = Instant::now();
    let f = typstyle.format_source(source);
    if args.pretty_doc {
        match f.render_ir() {
            Ok(ir) => println!("{ir}"),
            Err(e) => error!("Failed to render IR: {e}"),
        }
    }
    let Ok(res) = f.render() else {
        return FormatResult::Erroneous;
    };

    if args.timing {
        println!("Formatting completed in {:?}", start_time.elapsed());
    }

    // Compare `res` with `content` to perform CI checks
    if res != content {
        FormatResult::Formatted(res)
    } else {
        FormatResult::Unchanged
    }
}

fn get_input(input: Option<&Path>) -> Result<String> {
    match input {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display())),
        None => {
            let mut buffer = String::new();
            std::io::stdin()
                .read_to_string(&mut buffer)
                .with_context(|| "failed to read from stdin")?;
            Ok(buffer)
        }
    }
}

fn write_back(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content)
        .with_context(|| format!("failed to write to the file {}", path.display()))
}

fn resolve_typst_files(input: &[PathBuf]) -> Vec<PathBuf> {
    fn is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .is_some_and(|s| s.starts_with('.'))
    }

    let mut files = Vec::new();
    let mut has_dir = false;
    for path in input.iter().map(fs::normalize_path).unique() {
        if path.is_dir() {
            has_dir = true;
            let entries = WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| !is_hidden(e))
                .filter_map(Result::ok);
            for entry in entries {
                if entry.file_type().is_file() && entry.path().extension() == Some("typ".as_ref()) {
                    files.push(entry.into_path());
                }
            }
        } else {
            files.push(path.clone());
        }
    }
    if has_dir {
        files.sort_unstable();
    }
    files
}

fn print_unified_diff(original: &str, modified: &str, path: Option<&Path>) {
    print!(
        "{}",
        SourceDiff {
            original,
            modified,
            path,
        }
    );
}
