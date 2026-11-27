//! Style configuration for the Typstyle formatter.
//!
//! [`Options`] is the canonical option declaration. The [`options`] attribute generates
//! [`PartialOptions`], defaults, merging, validation, CLI arguments, and binding metadata.

use typstyle_macros::options;

use crate::option::OptionEnum;

/// Configuration options for the Typstyle printer.
#[options(partial = "PartialOptions")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Options {
    /// Number of spaces to use for each indentation level.
    #[option(default = 2)]
    #[cli(
        short = 't',
        long = "indent-width",
        visible_alias = "tab-width",
        value_name = "INDENT_WIDTH"
    )]
    #[ui(group = "Layout", label = "Indent width")]
    pub tab_spaces: usize,
    /// Maximum width of each line.
    #[option(default = 80)]
    #[cli(
        short = 'l',
        long = "line-width",
        visible_short_alias = 'c',
        visible_alias = "column",
        value_name = "LINE_WIDTH"
    )]
    #[ui(group = "Layout", widget = "number", label = "Line width")]
    pub max_width: usize,
    /// Maximum number of consecutive blank lines allowed.
    #[option(default = 1)]
    #[ui(group = "Layout")]
    pub blank_lines_upper_bound: usize,
    /// When `true`, consecutive whitespace in markup is collapsed into a single space.
    #[option(default = false)]
    #[ui(group = "Layout")]
    pub collapse_markup_spaces: bool,
    /// When `true`, import items are sorted alphabetically.
    #[option(default = true)]
    #[ui(group = "Imports")]
    pub reorder_import_items: bool,
    /// Policy for generated line breaks. Literal source text is preserved.
    #[option(default = LineEnding::Preserve, default_display = "preserve")]
    #[ui(group = "Layout")]
    pub line_ending: LineEnding,
    /// Text wrapping mode for markup.
    #[option(default = WrapMode::None, default_display = "none")]
    #[cli(
        long = "wrap-text",
        value_name = "WRAP_TEXT",
        num_args = 0..=1,
        default_missing_value = "fill",
        require_equals = true
    )]
    #[ui(group = "Layout")]
    pub wrap_mode: WrapMode,
}

/// Backwards-compatible alias for [`Options`].
pub type Config = Options;

/// Backwards-compatible alias for [`PartialOptions`].
pub type PartialConfig = PartialOptions;

/// Line-ending policy for generated layout, including partial formatting.
///
/// Literal text stays verbatim; raw-block boundary breaks follow the layout policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum LineEnding {
    /// A single line feed.
    Lf,
    /// A carriage return followed by a line feed.
    Crlf,
    /// Generate CRLF only if the entire source contains CRLF and no bare LF or CR.
    /// Sources without ASCII newlines or with mixed endings generate LF.
    #[default]
    Preserve,
}

/// Text wrapping mode for markup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum WrapMode {
    /// Do not wrap text.
    #[default]
    None,
    /// Wrap text to fit within the line width.
    Fill,
    /// Place each sentence on its own line.
    Sentence,
    /// Place each sentence on its own line, and wrap long sentences to the line width.
    FillSentence,
}

impl OptionEnum for LineEnding {
    const CHOICES: &'static [&'static str] = &["lf", "crlf", "preserve"];

    fn from_choice(value: &str) -> Option<Self> {
        match value {
            "lf" => Some(Self::Lf),
            "crlf" => Some(Self::Crlf),
            "preserve" => Some(Self::Preserve),
            _ => None,
        }
    }
}

impl OptionEnum for WrapMode {
    const CHOICES: &'static [&'static str] = &["none", "fill", "sentence", "fill-sentence"];

    fn from_choice(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "fill" => Some(Self::Fill),
            "sentence" => Some(Self::Sentence),
            "fill-sentence" => Some(Self::FillSentence),
            _ => None,
        }
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_exposed_by_the_macro() {
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
