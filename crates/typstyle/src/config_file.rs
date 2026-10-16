//! Discovery and parsing of `typstyle.toml` configuration files.
//!
//! The nearest configuration file is used as a complete replacement for any
//! configuration in parent directories: nothing is merged implicitly. The
//! user-facing schema in [`ConfigOverrides`] is deliberately separate from
//! [`Config`] so that keys follow the formatter's option names, and unknown or
//! ill-typed keys are rejected instead of ignored. The command line uses the
//! same schema, so both sources stay in sync.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use typstyle_core::{Config, LineEnding, WrapMode};

use crate::{
    cli::{ConfigArgs, StyleArgs},
    fs,
};

/// The name of the configuration file searched for in each directory.
const CONFIG_FILE_NAME: &str = "typstyle.toml";

/// A set of formatter option overrides, keyed like the configuration file.
///
/// Both a `typstyle.toml` file and the command line contribute overrides, and
/// the command line is applied last, so it wins. Every field is optional so
/// that an omitted option keeps the value from the other source or the
/// formatter default, and deserialization rejects unknown keys and values of
/// the wrong type.
///
/// Keys are the formatter's option names in kebab case. Two keys differ from
/// their command-line spellings: the wrap mode is `wrap-mode` although the flag
/// is `--wrap-text`, and import sorting is the positive `reorder-import-items`
/// although the flag is `--no-reorder-import-items`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ConfigOverrides {
    /// Maximum width of each line. The CLI flag is `--line-width`.
    line_width: Option<usize>,
    /// Number of spaces per indentation level.
    indent_width: Option<usize>,
    /// Maximum number of consecutive blank lines allowed.
    blank_lines_upper_bound: Option<usize>,
    /// When `true`, consecutive whitespace in markup is collapsed.
    collapse_markup_spaces: Option<bool>,
    /// When `true`, import items are sorted alphabetically.
    reorder_import_items: Option<bool>,
    /// Line-ending policy for formatted source output.
    line_ending: Option<LineEnding>,
    /// Text wrapping mode for markup. The CLI option is `--wrap-text`.
    wrap_mode: Option<WrapMode>,
}

impl ConfigOverrides {
    /// Parses the contents of a `typstyle.toml` file.
    fn parse(content: &str) -> Result<Self> {
        Ok(toml::from_str(content)?)
    }

    /// Overlays these overrides onto the given configuration.
    pub(crate) fn apply_to(&self, config: &mut Config) {
        if let Some(value) = self.line_width {
            config.max_width = value;
        }
        if let Some(value) = self.indent_width {
            config.tab_spaces = value;
        }
        if let Some(value) = self.blank_lines_upper_bound {
            config.blank_lines_upper_bound = value;
        }
        if let Some(value) = self.collapse_markup_spaces {
            config.collapse_markup_spaces = value;
        }
        if let Some(value) = self.reorder_import_items {
            config.reorder_import_items = value;
        }
        if let Some(value) = self.line_ending {
            config.line_ending = value;
        }
        if let Some(value) = self.wrap_mode {
            config.wrap_mode = value;
        }
    }
}

impl From<&StyleArgs> for ConfigOverrides {
    fn from(args: &StyleArgs) -> Self {
        Self {
            line_width: args.line_width,
            indent_width: args.indent_width,
            blank_lines_upper_bound: None,
            collapse_markup_spaces: None,
            // The flag is negative, the option and the file key are positive.
            reorder_import_items: args.no_reorder_import_items.map(|disabled| !disabled),
            line_ending: args.line_ending,
            wrap_mode: args.wrap_text,
        }
    }
}

/// Resolves the `typstyle.toml` configuration of each input of a CLI run.
///
/// Discovery results and parsed files are cached, so formatting many inputs
/// does not read the same configuration file repeatedly.
pub struct ConfigResolver {
    /// The `--config` path, used for every input instead of discovery.
    explicit: Option<PathBuf>,
    /// Whether `--isolated` disables configuration files entirely.
    isolated: bool,
    /// The `--stdin-filename` path, seeding discovery for stdin input.
    stdin_path: Option<PathBuf>,
    /// Discovered configuration paths, keyed by the directory to start from.
    discovered: HashMap<PathBuf, Option<PathBuf>>,
    /// Parsed configuration files, keyed by configuration path.
    parsed: HashMap<PathBuf, Config>,
}

impl ConfigResolver {
    /// Creates a resolver from the configuration options of the CLI.
    ///
    /// An explicit `--config` file is read and parsed immediately, so an
    /// unusable configuration file is reported before any input is processed.
    pub fn new(args: &ConfigArgs) -> Result<Self> {
        let mut resolver = Self {
            explicit: args.config.as_deref().map(fs::normalize_path),
            isolated: args.isolated,
            stdin_path: args.stdin_filename.as_deref().map(fs::normalize_path),
            discovered: HashMap::new(),
            parsed: HashMap::new(),
        };
        if let Some(explicit) = resolver.explicit.clone() {
            resolver.load(&explicit)?;
        }
        Ok(resolver)
    }

    /// Returns the configuration file's contribution for the given input.
    ///
    /// `input` is the file being formatted, or `None` for stdin, which seeds
    /// discovery from the directory of `--stdin-filename` when it is given.
    pub fn config_for(&mut self, input: Option<&Path>) -> Result<Config> {
        if self.isolated {
            return Ok(Config::default());
        }
        let Some(path) = self.config_path_for(input)? else {
            return Ok(Config::default());
        };
        self.load(&path)
    }

    /// Returns the configuration file selected for the given input, if any.
    fn config_path_for(&mut self, input: Option<&Path>) -> Result<Option<PathBuf>> {
        if let Some(explicit) = &self.explicit {
            return Ok(Some(explicit.clone()));
        }
        let start = match input.and_then(Path::parent).or(self.stdin_path.as_deref()) {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => std::env::current_dir().context("failed to get current directory")?,
        };
        let start = fs::normalize_path(start);
        if let Some(found) = self.discovered.get(&start) {
            return Ok(found.clone());
        }
        let found = find_config_file(&start);
        self.discovered.insert(start, found.clone());
        Ok(found)
    }

    /// Reads, parses, and caches the configuration file at the given path.
    fn load(&mut self, path: &Path) -> Result<Config> {
        if let Some(config) = self.parsed.get(path) {
            return Ok(config.clone());
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let overrides = ConfigOverrides::parse(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        let mut config = Config::default();
        overrides.apply_to(&mut config);
        self.parsed.insert(path.to_path_buf(), config.clone());
        Ok(config)
    }
}

/// Searches the given directory and its ancestors for the nearest
/// `typstyle.toml` file.
fn find_config_file(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .find(|candidate| candidate.is_file())
}
