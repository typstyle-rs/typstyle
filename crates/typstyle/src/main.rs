mod cli;
mod diff;
mod fmt;
mod fs;
mod logging;

use std::{io::Write, process::ExitCode};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use fmt::{format, format_stdin};

use crate::cli::CliArguments;

#[derive(Copy, Clone)]
pub enum ExitStatus {
    /// Execution was successful and there were no errors.
    Success,
    /// Execution was successful but there were errors.
    Failure,
    /// Execution failed.
    Error,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => ExitCode::from(0),
            ExitStatus::Failure => ExitCode::from(1),
            ExitStatus::Error => ExitCode::from(2),
        }
    }
}

fn main() -> ExitCode {
    let args = CliArguments::parse();
    args.validate_input();

    logging::init();
    log::set_max_level(if args.log_level.verbose {
        log::LevelFilter::Debug
    } else if args.log_level.quiet {
        log::LevelFilter::Error
    } else {
        log::LevelFilter::Info
    });

    match execute(args) {
        Ok(code) => code.into(),
        Err(err) => {
            let mut stderr = std::io::stderr().lock();
            for cause in err.chain() {
                writeln!(stderr, "  {} {cause}", "Cause:".bold()).ok();
            }
            ExitStatus::Error.into()
        }
    }
}

fn execute(args: CliArguments) -> Result<ExitStatus> {
    #[cfg(feature = "completion")]
    if let Some(command) = &args.command {
        match command {
            cli::Command::Completions { shell } => {
                use clap::CommandFactory;

                clap_complete::generate(
                    *shell,
                    &mut cli::CliArguments::command(),
                    "typstyle",
                    &mut std::io::stdout(),
                );

                return Ok(ExitStatus::Success);
            }
        }
    }

    if args.input.is_empty() {
        format_stdin(&args)
    } else {
        format(&args)
    }
}
