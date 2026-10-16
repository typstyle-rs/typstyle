//! Definition of the `typstyle` command-line interface, parsed with clap.
//!
//! [`StyleArgs`] maps the style options to the formatter's [`typstyle_core::Config`].

use std::{path::PathBuf, sync::LazyLock};

use clap::{Args, CommandFactory, Parser, Subcommand, error::ErrorKind};
use typstyle_core::{LineEnding, WrapMode};

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

    /// Options selecting which configuration file applies.
    #[command(flatten, next_help_heading = "Configuration File")]
    pub config: ConfigArgs,

    /// Options for printing debug output of the formatter.
    #[command(flatten, next_help_heading = "Debug Options")]
    pub debug: DebugArgs,

    /// Options controlling the logging verbosity.
    #[command(flatten, next_help_heading = "Log Levels")]
    pub log_level: LogLevelArgs,
}

impl CliArguments {
    /// Exits with an error if in-place formatting is requested without any input files.
    pub fn validate_input(&self) {
        if self.command.is_none() && self.inplace && self.input.is_empty() {
            let mut cmd = Self::command();
            cmd.error(
                ErrorKind::ValueValidation,
                "cannot perform in-place formatting without at least one file being presented",
            )
            .exit();
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
///
/// Every option is optional, so that options omitted on the command line leave
/// the value selected by the configuration file in place.
#[derive(Args)]
pub struct StyleArgs {
    /// Maximum width of each line [default: 80]
    #[arg(
        short = 'l',
        long,
        visible_short_alias = 'c',
        visible_alias = "column",
        global = true
    )]
    pub line_width: Option<usize>,

    /// Number of spaces per indentation level [default: 2]
    #[arg(short = 't', long, visible_alias = "tab-width", global = true)]
    pub indent_width: Option<usize>,

    /// Line-ending policy for formatted source output [default: preserve]
    #[arg(long, value_enum, global = true)]
    pub line_ending: Option<LineEnding>,

    /// Disable alphabetical reordering of import items.
    #[arg(
        long,
        value_name = "BOOL",
        num_args = 0..=1,
        default_missing_value = "true",
        require_equals = true,
        hide_possible_values = true,
        global = true
    )]
    pub no_reorder_import_items: Option<bool>,

    /// Text wrapping mode: none, fill (wrap to line width), sentence (one per line), or fill-sentence (both) [default: none]
    #[arg(
        long,
        value_enum,
        default_missing_value = "fill",
        num_args = 0..=1,
        require_equals = true,
        global = true
    )]
    pub wrap_text: Option<WrapMode>,
}

/// Options selecting which `typstyle.toml` configuration applies.
#[derive(Args)]
pub struct ConfigArgs {
    /// Path to a `typstyle.toml` file to use for every input
    #[arg(long, value_name = "PATH", global = true, conflicts_with = "isolated")]
    pub config: Option<PathBuf>,

    /// Ignore `typstyle.toml` files and use the default configuration
    #[arg(long, default_value_t = false, global = true)]
    pub isolated: bool,

    /// Name of the file being formatted, used to discover the configuration
    /// when reading from stdin
    #[arg(long, value_name = "PATH", global = true)]
    pub stdin_filename: Option<PathBuf>,
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
