//! Configuration discovery and layering for the CLI.
//!
//! Resolution order, from lowest to highest priority:
//!
//! 1. built-in defaults,
//! 2. the user configuration file,
//! 3. the project configuration file (or `--config`),
//! 4. explicit command-line overrides.
//!
//! Validation runs only after all layers are merged.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use typstyle_core::{Options, PartialOptions};

use crate::cli::CliArguments;

/// Resolves the effective formatter configuration for a CLI invocation.
pub fn resolve(args: &CliArguments) -> Result<Options> {
    let mut overrides = PartialOptions::default();

    if let Some(path) = user_config_path()
        && path.is_file()
    {
        overrides = overrides.merge(read(&path)?);
    }

    let project = match &args.config {
        Some(path) => Some(path.clone()),
        None => find_project_config(),
    };
    if let Some(path) = project {
        overrides = overrides.merge(read(&path)?);
    }

    overrides = overrides.merge(args.style.to_partial());
    overrides
        .resolve()
        .map_err(|error| anyhow!("invalid formatter configuration: {error}"))
}

/// Reads and deserializes one TOML configuration file.
fn read(path: &Path) -> Result<PartialOptions> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read configuration file {}", path.display()))?;
    toml::from_str(&text)
        .with_context(|| format!("failed to parse configuration file {}", path.display()))
}

/// Returns the user-level configuration path, if one is configured.
fn user_config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("TYPSTYLE_CONFIG") {
        return Some(PathBuf::from(path));
    }
    if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config).join("typstyle/config.toml"));
    }
    if cfg!(windows) {
        return env::var_os("APPDATA")
            .map(|path| PathBuf::from(path).join("typstyle").join("config.toml"));
    }
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/typstyle/config.toml"))
}

/// Finds the closest project configuration file, walking up from the current directory.
fn find_project_config() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        for name in [".typstyle.toml", "typstyle.toml"] {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}
