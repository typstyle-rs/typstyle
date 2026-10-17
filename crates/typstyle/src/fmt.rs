/// This module provides functionality to format Typst files either in-place or by checking
/// their formatting via standard input/output.
///
/// Adapted from: https://github.com/astral-sh/ruff/blob/main/crates/ruff_linter/src/fs.rs
use std::{
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use itertools::Itertools;
use log::{debug, error, info, warn};
use typst_syntax::Source;
use typstyle_core::{Config, LineEnding, Typstyle, WrapMode, format_ast};

use crate::{
    ExitStatus,
    cli::{CliArguments, DebugArgs, FileSelectionArgs, LineEndingMode, StyleArgs, WrapTextMode},
    diff::SourceDiff,
    fs,
};

/// The mode in which the formatter reports and writes results.
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
    /// Determines the mode from the CLI arguments: check wins over diff.
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
    /// Converts the style options into the formatter's [`Config`].
    pub fn to_config(&self) -> Config {
        Config {
            max_width: self.line_width,
            tab_spaces: self.indent_width,
            line_ending: match self.line_ending {
                LineEndingMode::Lf => LineEnding::Lf,
                LineEndingMode::Crlf => LineEnding::Crlf,
                LineEndingMode::Preserve => LineEnding::Preserve,
            },
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

/// Formats the input from stdin, returning the resulting exit status.
///
/// In check or diff mode, returns [`ExitStatus::Failure`] if the input would
/// be reformatted.
pub fn format_stdin(args: &CliArguments) -> Result<ExitStatus> {
    let typstyle = Typstyle::new(args.style.to_config());

    format_one(None, &typstyle, args).map(|res| match res {
        FormatResult::Formatted(_) if args.check || args.diff => ExitStatus::Failure,
        _ => ExitStatus::Success,
    })
}

/// Formats all Typst files under the given input paths.
///
/// Logs a summary of the run and returns [`ExitStatus::Failure`] in check or
/// diff mode if any file would be reformatted.
pub fn format(args: &CliArguments) -> Result<ExitStatus> {
    #[derive(Default)]
    struct Summary {
        format_count: usize,
        unchanged_count: usize,
        error_count: usize,
    }
    let mut summary = Summary::default();

    let mode = FormatMode::from_cli(args);
    // Discovery runs to completion before any file is formatted, so the errors
    // reported by the walker surface before any write instead of producing a
    // partial run. The walker suppresses some ignore-file errors internally.
    let paths = resolve_typst_files(&args.input, &args.file_selection)?;
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

/// Formats a single `.typ` file, or stdin if no path is given.
///
/// Depending on the mode, prints the result to stdout, writes it back to the
/// file in place, reports it in check mode, or prints a unified diff. Unchanged
/// and erroneous inputs are echoed unchanged to stdout for piping.
fn format_one(
    input: Option<&Path>,
    typstyle: &Typstyle,
    args: &CliArguments,
) -> Result<FormatResult> {
    let use_stdout = !args.inplace && !args.check && !args.diff;
    let unformatted = get_input(input)?;

    let res = format_debug(&unformatted, typstyle, &args.debug);
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

/// The outcome of formatting one input.
enum FormatResult {
    /// The formatted content, which differs from the original.
    Formatted(String),
    /// The input is already formatted.
    Unchanged,
    /// The input cannot be parsed and was left as is.
    Erroneous,
}

/// Formats one source while printing the debug output selected by the options.
///
/// Prints the AST, the pretty IR, and the elapsed time if requested, and
/// reports [`FormatResult::Erroneous`] if the source cannot be rendered.
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

/// Reads the input from the given file, or from stdin if no path is given.
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

/// Writes the formatted content back to the file.
fn write_back(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content)
        .with_context(|| format!("failed to write to the file {}", path.display()))
}

/// Resolves the input paths to the files to format.
///
/// Directories are searched recursively for `.typ` files. Entries whose name
/// starts with a dot are skipped, and the standard ignore files are consulted
/// unless the selection options opt out. Explicitly named files are always
/// kept, even when they are hidden, ignored, or use another extension.
/// Duplicates are removed, and the results are sorted if any directory was
/// expanded.
///
/// Ignore matching follows the ignore patterns of the repository; it never
/// consults Git's tracked-file index, so a tracked file matching an ignore
/// pattern is skipped during recursive discovery.
fn resolve_typst_files(input: &[PathBuf], selection: &FileSelectionArgs) -> Result<Vec<PathBuf>> {
    fn is_hidden(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
    }

    let mut roots = Vec::new();
    let mut files = Vec::new();
    for path in input.iter().map(fs::normalize_path).unique() {
        if path.is_dir() {
            roots.push(path);
        } else {
            files.push(path);
        }
    }

    let has_dir = !roots.is_empty();
    if has_dir {
        let mut builder = WalkBuilder::from_iter(roots);
        builder
            // Hidden entries are handled by the predicate below so that they
            // stay skipped regardless of ignore-file negations or the opt-out.
            .hidden(false)
            .filter_entry(|entry| entry.depth() == 0 || !is_hidden(entry.path()));
        let respect = selection.respect_ignore_files();
        builder
            .ignore(respect)
            .git_ignore(respect)
            .git_global(respect)
            .git_exclude(respect);
        for entry in builder.build() {
            let entry = entry
                .with_context(|| "failed to discover files, check paths and their permissions")?;
            // Errors the walker reports carry the path of the entry they were
            // found in; surface them instead of skipping the entry silently.
            if let Some(err) = entry.error() {
                return Err(anyhow::anyhow!(err.to_string())).with_context(|| {
                    format!("failed to read ignore files for {}", entry.path().display())
                });
            }
            if entry.file_type().is_some_and(|t| t.is_file())
                && entry.path().extension() == Some("typ".as_ref())
            {
                files.push(entry.into_path());
            }
        }
    }
    if has_dir {
        files.sort_unstable();
    }
    Ok(files.into_iter().unique().collect())
}

/// Prints the unified diff of the changes made to the given path.
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
