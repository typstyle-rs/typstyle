//! Definition of the `typstyle` command-line interface, parsed with clap.
//!
//! Style arguments are generated from `options.toml` by
//! `cargo xtask generate-options`. Only CLI concerns that do not describe an option remain
//! handwritten.

use std::{path::PathBuf, sync::LazyLock};

use clap::{Args, CommandFactory, Parser, Subcommand, error::ErrorKind};

use crate::fmt::LineRange;
use crate::generated_args::OptionArgs;

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

    /// Path to a TOML configuration file.
    ///
    /// When omitted, `typstyle.toml` or `.typstyle.toml` is discovered from the current
    /// directory upwards, and a user configuration is read from
    /// `$TYPSTYLE_CONFIG`, `$XDG_CONFIG_HOME/typstyle/config.toml`, or
    /// `~/.config/typstyle/config.toml`.
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help_heading = "Configuration"
    )]
    pub config: Option<PathBuf>,

    /// Options configuring the formatting style.
    #[command(flatten, next_help_heading = "Format Configuration")]
    pub style: OptionArgs,

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
