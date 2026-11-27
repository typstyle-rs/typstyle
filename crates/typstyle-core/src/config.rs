//! Style configuration for the Typstyle formatter.
//!
//! [`Options`], [`PartialOptions`], and the option enums are generated from
//! `options.toml` by `cargo xtask generate-options`. Add declarations to that file and
//! regenerate the bindings rather than editing the generated module.

#[rustfmt::skip]
mod generated;

pub use generated::{LineEnding, Options, PartialOptions, WrapMode};

use crate::option::OptionsError;

/// Backwards-compatible alias for [`Options`].
pub type Config = Options;

/// Backwards-compatible alias for [`PartialOptions`].
pub type PartialConfig = PartialOptions;

impl LineEnding {
    /// Resolve source-dependent policy against the whole original document.
    pub(crate) fn resolve(self, content: &str) -> Self {
        if self != Self::Preserve {
            return self;
        }
        let bytes = content.as_bytes();
        let mut saw_crlf = false;
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                    saw_crlf = true;
                    index += 2;
                }
                b'\r' | b'\n' => return Self::Lf,
                _ => index += 1,
            }
        }
        if saw_crlf { Self::Crlf } else { Self::Lf }
    }
}

impl Options {
    /// Creates a configuration with the default values (see [`Options::default`]).
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the maximum width of each line ([`Options::max_width`]).
    pub fn with_width(mut self, max_width: usize) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the number of spaces per indentation level ([`Options::tab_spaces`]).
    pub fn with_tab_spaces(mut self, tab_spaces: usize) -> Self {
        self.tab_spaces = tab_spaces;
        self
    }

    /// Returns the maximum width for keeping field-access chains on one line.
    ///
    /// Chains whose estimated length reaches this bound (a fraction of
    /// [`Options::max_width`]) are broken across lines.
    pub fn chain_width(&self) -> usize {
        const CHAIN_WIDTH_RATIO: f32 = 0.6;
        (self.max_width as f32 * CHAIN_WIDTH_RATIO) as usize
    }

    /// Sets the terminator for generated layout ([`Options::line_ending`]).
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }

    /// Resolves source-dependent options against the entire original document.
    ///
    /// Low-level [`crate::pretty::PrettyPrinter`] callers should use this before
    /// constructing a printer. [`crate::Typstyle`] resolves policies automatically,
    /// including when formatting a range within a document.
    pub fn for_source(mut self, source: &str) -> Self {
        self.line_ending = self.line_ending.resolve(source);
        self
    }

    /// Sets whether markup text is wrapped: `true` selects [`WrapMode::Fill`],
    /// `false` selects [`WrapMode::None`].
    pub fn with_wrap_text(mut self, wrap_text: bool) -> Self {
        self.wrap_mode = if wrap_text {
            WrapMode::Fill
        } else {
            WrapMode::None
        };
        self
    }

    /// Sets the text wrapping mode ([`Options::wrap_mode`]).
    pub fn with_wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
        self.wrap_mode = wrap_mode;
        self
    }

    /// Validates cross-field constraints.
    ///
    /// Field types and enum values are enforced by deserialization. Constraints that span
    /// multiple fields belong here; the generated `apply` method calls this after merging.
    pub fn validate(&self) -> Result<(), OptionsError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_exposed_by_generated_code() {
        let config = Options::default();
        assert_eq!(config.tab_spaces, 2);
        assert_eq!(config.max_width, 80);
        assert_eq!(config.blank_lines_upper_bound, 1);
        assert!(!config.collapse_markup_spaces);
        assert!(config.reorder_import_items);
        assert_eq!(config.line_ending, LineEnding::Preserve);
        assert_eq!(config.wrap_mode, WrapMode::None);
    }

    #[test]
    fn newer_overrides_win_and_explicit_false_is_preserved() {
        let merged = PartialOptions {
            max_width: Some(100),
            reorder_import_items: Some(false),
            ..Default::default()
        }
        .merge(PartialOptions {
            max_width: Some(120),
            ..Default::default()
        });
        let resolved = merged.resolve().unwrap();
        assert_eq!(resolved.max_width, 120);
        assert!(!resolved.reorder_import_items);
    }

    #[test]
    fn option_specs_cover_every_declared_field() {
        let keys = Options::OPTION_SPECS
            .iter()
            .map(|spec| spec.key)
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            [
                "tab_spaces",
                "max_width",
                "blank_lines_upper_bound",
                "collapse_markup_spaces",
                "reorder_import_items",
                "line_ending",
                "wrap_mode",
            ]
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn wrap_mode_serde_round_trips_through_kebab_case() {
        for (mode, json) in [
            (WrapMode::None, "\"none\""),
            (WrapMode::Fill, "\"fill\""),
            (WrapMode::Sentence, "\"sentence\""),
            (WrapMode::FillSentence, "\"fill-sentence\""),
        ] {
            assert_eq!(serde_json::to_string(&mode).unwrap(), json);
            assert_eq!(serde_json::from_str::<WrapMode>(json).unwrap(), mode);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn partial_config_rejects_unknown_keys() {
        let error = serde_json::from_str::<PartialOptions>(r#"{"unknown": 1}"#).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialized_overrides_omit_unset_fields() {
        let value = serde_json::to_value(PartialOptions {
            max_width: Some(100),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(value, serde_json::json!({ "max_width": 100 }));
    }
}
