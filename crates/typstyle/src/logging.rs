//! Initialization of the CLI's colored logging.

use colored::Colorize;
use log::{Level, LevelFilter, Metadata, Record};

/// A minimal logger that prints messages with colored error and warning prefixes.
///
/// Errors and warnings go to stderr; other levels are printed to stdout.
pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    /// Returns whether records at the given level are currently logged.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    /// Prints the record, colorizing error and warning levels.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => {
                    anstream::eprintln!("{}{} {}", "error".bold().red(), ":".bold(), record.args())
                }
                Level::Warn => anstream::eprintln!(
                    "{}{} {}",
                    "warn".bold().yellow(),
                    ":".bold(),
                    record.args()
                ),
                _ => anstream::println!("{}", record.args()),
            }
        }
    }

    /// Does nothing; output is not buffered.
    fn flush(&self) {}
}

/// Installs [`SimpleLogger`] as the global logger with the default `Info` level.
///
/// Panics if a logger is already set.
pub fn init() {
    log::set_logger(&SimpleLogger)
        .map(|_| log::set_max_level(LevelFilter::Info))
        .expect("failed to init logging");
}
