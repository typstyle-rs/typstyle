//! Definition of the `typstyle` command-line interface, parsed with clap.
//!
//! [`StyleArgs`] maps the style options to the formatter's [`typstyle_core::Config`].

use std::{path::PathBuf, sync::LazyLock};

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum, error::ErrorKind};

use crate::fmt::LineRange;

/// Arguments of the `typstyle` CLI.
#[derive(Parser)]
#[command(
  name = "typstyle",
  about = "Beautiful and reliable typst code formatter",
  author, version, long_version(LONG_VERSION.as_str())
)]
pub struct CliArguments {
    /// The subcommand to run, if any.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// List of files or directories to format [default: stdin]
    pub input: Vec<PathBuf>,

    /// Format only the given line ranges, using 1-based inclusive line numbers.
    /// Repeat the option to select multiple ranges.
    #[arg(
        long,
        value_name = "FIRST:LAST",
        num_args = 1,
        help_heading = "Range Selection",
        conflicts_with = "pretty_doc"
    )]
    pub lines: Vec<LineRange>,

    /// Format the file in place
    #[arg(short, long, default_value_t = false, conflicts_with_all = ["check", "diff"])]
    pub inplace: bool,

    /// Run in 'check' mode. Exits with 0 if input is formatted correctly.
    /// Exits with a non-zero status code if formatting is required.
    #[arg(long, default_value_t = false, global = true, conflicts_with = "diff")]
    pub check: bool,

    /// Run in 'diff' mode. Shows unified diff of what formatting changes would be made.
    /// Exits with 0 if input is formatted correctly. Exits with a non-zero status code if formatting is required.
    #[arg(long, default_value_t = false, global = true, conflicts_with = "check")]
    pub diff: bool,

    /// Options configuring the formatting style.
    #[command(flatten, next_help_heading = "Format Configuration")]
    pub style: StyleArgs,

    /// Options for printing debug output of the formatter.
    #[command(flatten, next_help_heading = "Debug Options")]
    pub debug: DebugArgs,

    /// Options controlling the logging verbosity.
    #[command(flatten, next_help_heading = "Log Levels")]
    pub log_level: LogLevelArgs,
}

impl CliArguments {
    /// Exits with an error if the selected options cannot be combined.
    pub fn validate_input(&self) {
        if self.command.is_some() {
            return;
        }
        if self.inplace && self.input.is_empty() {
            Self::command()
                .error(
                    ErrorKind::ValueValidation,
                    "cannot perform in-place formatting without at least one file being presented",
                )
                .exit();
        }
        if !self.lines.is_empty() {
            if self.input.len() > 1 {
                Self::command()
                    .error(
                        ErrorKind::ValueValidation,
                        "`--lines` can only be used with stdin or a single input",
                    )
                    .exit();
            }
            if self.input.first().is_some_and(|path| path.is_dir()) {
                Self::command()
                    .error(
                        ErrorKind::ValueValidation,
                        "`--lines` cannot be used with a directory input",
                    )
                    .exit();
            }
        }
    }
}

/// Subcommands of the `typstyle` CLI.
#[derive(Subcommand)]
pub enum Command {
    #[cfg(feature = "completion")]
    /// Generate shell completions for the given shell to stdout
    #[command(hide = true)]
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

/// Style options that map to the formatter's [`typstyle_core::Config`].
#[derive(Args)]
pub struct StyleArgs {
    /// Maximum width of each line.
    #[arg(
        short = 'l',
        long,
        visible_short_alias = 'c',
        visible_alias = "column",
        default_value_t = 80,
        global = true
    )]
    pub line_width: usize,

    /// Number of spaces per indentation level.
    #[arg(
        short = 't',
        long,
        visible_alias = "tab-width",
        default_value_t = 2,
        global = true
    )]
    pub indent_width: usize,

    /// Line-ending policy for formatted source output.
    #[arg(long, value_enum, default_value_t = LineEndingMode::Preserve, global = true)]
    pub line_ending: LineEndingMode,

    /// Disable alphabetical reordering of import items.
    #[arg(long, default_value_t = false, global = true)]
    pub no_reorder_import_items: bool,

    /// Text wrapping mode: none (default), fill (wrap to line width), sentence (one per line), or fill-sentence (both).
    #[arg(
        long,
        value_enum,
        default_value_t = WrapTextMode::None,
        default_missing_value = "fill",
        num_args = 0..=1,
        require_equals = true,
        global = true
    )]
    pub wrap_text: WrapTextMode,
}

/// CLI policy for selecting the generated line terminator for each input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum LineEndingMode {
    /// Generate LF breaks, preserving literal source text.
    Lf,
    /// Generate CRLF breaks, preserving literal source text.
    Crlf,
    /// Generate CRLF only when the input contains CRLF and no bare LF or CR.
    #[default]
    Preserve,
}

/// Text wrapping mode for CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum WrapTextMode {
    /// Do not wrap text (default)
    #[default]
    None,
    /// Wrap text to fit within line width
    Fill,
    /// Place each sentence on its own line
    Sentence,
    /// Place each sentence on its own line, wrapping long sentences to the line width
    FillSentence,
}

/// Debug options for printing intermediate artifacts of the formatter.
#[derive(Args)]
pub struct DebugArgs {
    /// Print the AST of the input file
    #[arg(short, long, default_value_t = false)]
    pub ast: bool,

    /// Print the pretty document
    #[arg(short, long, default_value_t = false)]
    pub pretty_doc: bool,

    /// Show elapsed time taken by the formatter
    #[arg(long, default_value_t = false)]
    pub timing: bool,
}

/// Options controlling how much is logged.
#[derive(Args)]
pub struct LogLevelArgs {
    /// Enable verbose logging.
    #[arg(short, long, global = true, group = "verbosity")]
    pub verbose: bool,
    /// Print diagnostics, but nothing else.
    #[arg(short, long, global = true, group = "verbosity")]
    pub quiet: bool,
}

static LONG_VERSION: LazyLock<String> = LazyLock::new(|| {
    const NONE: &str = "None";
    format!(
        "
Version:             {}
Build Timestamp:     {}
Build Git Describe:  {}
Commit SHA:          {}
Commit Date:         {}
Commit Branch:       {}
Cargo Target Triple: {}
",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_BUILD_TIMESTAMP"),
        option_env!("VERGEN_GIT_DESCRIBE").unwrap_or(NONE),
        option_env!("VERGEN_GIT_SHA").unwrap_or(NONE),
        option_env!("VERGEN_GIT_COMMIT_TIMESTAMP").unwrap_or(NONE),
        option_env!("VERGEN_GIT_BRANCH").unwrap_or(NONE),
        env!("VERGEN_CARGO_TARGET_TRIPLE"),
    )
});
