use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::{Options, read_content};

/// Parses typstyle directives from consecutive directive lines at the start of a file
///
/// ```typst
/// /// typstyle: reorder-import-items=true max-width=100
/// /// typstyle: wrap-text=false
/// /// typstyle: include=/shared/common-helpers.typ
/// /// typstyle: include=relative-file.typ
///
/// #import "module.typ": a, b
///
/// // The included files will be appended to the end of this content
/// ```
pub fn parse_directives(content: &str) -> Result<Options> {
    let mut options = Options::default();

    // Process all consecutive directive lines at the start
    for line in content.lines() {
        // Check if the line starts with the directive marker
        if let Some(directive_content) = line.trim_start().strip_prefix("/// typstyle:") {
            // Split by spaces to get individual directives
            for directive in directive_content.split_whitespace() {
                // Check if it's a key-value pair
                let (key, value) = directive
                    .split_once('=')
                    .map(|(key, value)| (key.trim(), Some(value.trim())))
                    .unwrap_or((directive, None));
                // Normalize key by converting hyphens to underscores for simpler matching
                let key = key.replace('-', "_");
                update_options(&mut options, &key, value)?;
            }
        } else if line.trim().is_empty() {
            // Skip empty lines at the start
            continue;
        } else {
            // Stop at the first non-directive, non-empty line
            break;
        }
    }

    fn update_options(options: &mut Options, key: &str, value: Option<&str>) -> Result<()> {
        let config = &mut options.config;
        match key {
            "relax_convergence" => {
                options.relax_convergence = value.and_then(|v| v.parse().ok()).unwrap_or(1);
            }
            "include" => {
                if let Some(include_spec) = value {
                    // Store the original include specification as string
                    options.include_specs.push(include_spec.to_string());
                } else {
                    bail!("include directive requires a value");
                }
            }
            // Configuration options
            "reorder_import_items" => {
                config.reorder_import_items = value != Some("false");
            }
            "wrap_text" => {
                config.wrap_text = value != Some("false");
            }
            "collapse_markup_spaces" => {
                config.collapse_markup_spaces = value != Some("false");
            }
            "max_width" => {
                if let Some(v) = value {
                    config.max_width = v
                        .parse()
                        .with_context(|| format!("Invalid max_width value: {v}"))?;
                }
            }
            "tab_spaces" => {
                if let Some(v) = value {
                    config.tab_spaces = v
                        .parse()
                        .with_context(|| format!("Invalid tab_spaces value: {v}"))?;
                }
            }
            _ => bail!("unknown directive: {key}"),
        }
        Ok(())
    }

    // Apply implied settings
    options.config.collapse_markup_spaces |= options.config.wrap_text;

    Ok(options)
}

/// Resolves an include path specification to an absolute path
fn resolve_include_path(include_spec: &str, base_path: &Path) -> Result<PathBuf> {
    use crate::common::fixtures_dir;

    if include_spec.starts_with('/') {
        // Path starts with '/', treat as relative to fixtures directory
        Ok(fixtures_dir().join(include_spec.trim_start_matches('/')))
    } else {
        // Path doesn't start with '/', treat as relative to the parent directory of the current file
        let parent = base_path
            .parent()
            .context("Cannot determine parent directory of the current file")?;
        Ok(parent.join(include_spec))
    }
}

/// Processes include directives by appending included content to the end of the original content
pub fn process_includes(base_path: &Path, content: &str, options: &Options) -> Result<String> {
    if options.include_specs.is_empty() {
        return Ok(content.to_string());
    }

    let mut result = content.to_string();

    // Process each included file sequentially
    for include_spec in &options.include_specs {
        // Resolve the path when processing includes
        let resolved_path = resolve_include_path(include_spec, base_path)?;

        let included_content = read_content(&resolved_path).with_context(|| {
            format!("Failed to read include file '{}'", resolved_path.display())
        })?;
        // Append the included content to the result
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&included_content);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::bool_assert_comparison)]
    #![allow(unused)]

    use typstyle_core::Config;

    use super::*;
    use crate::common::fixtures_dir;

    #[test]
    fn parse_single_directive() {
        let content = "
        /// typstyle: wrap-text
        #import \"module.typ\": a, b";
        let options = parse_directives(content).unwrap();

        assert_eq!(
            options,
            Options {
                config: Config {
                    wrap_text: true,
                    collapse_markup_spaces: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        );
    }

    #[test]
    fn parse_multiple_directive_lines() {
        let content = r#"
/// typstyle: reorder-import-items=true
/// typstyle: max-width=40 relax_convergence=2
/// typstyle: include=test.typ

#import "module.typ": a, b"#;

        let options = parse_directives(content).unwrap();

        assert_eq!(
            options,
            Options {
                config: Config {
                    max_width: 40,
                    ..Default::default()
                },
                relax_convergence: 2,
                include_specs: vec!["test.typ".to_string()],
                ..Default::default()
            }
        );
    }

    #[test]
    fn parse_multiple_includes() {
        let content = r#"
/// typstyle: include=/shared/helpers.typ
/// typstyle: include=local.typ
/// typstyle: reorder-import-items=true

#import "module.typ": a, b"#;

        let options = parse_directives(content).unwrap();
        assert_eq!(
            options,
            Options {
                config: Config {
                    reorder_import_items: true,
                    ..Default::default()
                },
                include_specs: vec!["/shared/helpers.typ".to_string(), "local.typ".to_string()],
                ..Default::default()
            }
        );
    }

    #[test]
    fn parse_directive_lines_with_empty_lines() {
        let content = r#"
/// typstyle: reorder-import-items

/// typstyle: collapse-markup-spaces
/// typstyle: relax_convergence=3

#import "module.typ": a, b"#;

        let options = parse_directives(content).unwrap();
        assert_eq!(
            options,
            Options {
                config: Config {
                    collapse_markup_spaces: true,
                    wrap_text: false,
                    ..Default::default()
                },
                relax_convergence: 3,
                ..Default::default()
            }
        );
    }

    #[test]
    fn stop_at_non_directive_line() {
        let content = r#"
/// typstyle: reorder-import-items=true
// This is a regular comment, not a directive
/// typstyle: wrap-text

#import "module.typ": a, b"#;

        let options = parse_directives(content).unwrap();
        assert_eq!(
            options,
            Options {
                config: Config {
                    reorder_import_items: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        );
    }
}
