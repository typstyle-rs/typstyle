//! Style configuration for the Typstyle formatter.

/// Configuration Options for Typstyle Printer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Config {
    /// Number of spaces to use for each indentation level.
    pub tab_spaces: usize,
    /// Maximum width of each line.
    pub max_width: usize,
    /// Maximum number of consecutive blank lines allowed.
    pub blank_lines_upper_bound: usize,
    /// When `true`, consecutive whitespace in markup is collapsed into a single space.
    pub collapse_markup_spaces: bool,
    /// When `true`, import items are sorted alphabetically.
    pub reorder_import_items: bool,
    /// Policy for generated line breaks. Literal source text is preserved.
    pub line_ending: LineEnding,
    /// Text wrapping mode for markup.
    pub wrap_mode: WrapMode,
}

/// Line-ending policy for generated layout, including partial formatting.
///
/// Literal text stays verbatim; raw-block boundary breaks follow the layout
/// policy. `Preserve` resolves against the whole original document: it generates
/// CRLF only if the source contains CRLF and no bare LF or CR, and LF otherwise,
/// including for sources without ASCII newlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum LineEnding {
    /// Generate LF breaks, preserving literal source text.
    Lf,
    /// Generate CRLF breaks, preserving literal source text.
    Crlf,
    /// Generate CRLF only when the input contains CRLF and no bare LF or CR.
    #[default]
    Preserve,
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

/// Text wrapping mode for markup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_spaces: 2,
            max_width: 80,
            blank_lines_upper_bound: 1,
            reorder_import_items: true,
            collapse_markup_spaces: false,
            line_ending: LineEnding::Preserve,
            wrap_mode: WrapMode::None,
        }
    }
}

impl Config {
    /// Creates a configuration with the default values (see [`Config::default`]).
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the maximum width of each line ([`Config::max_width`]).
    pub fn with_width(mut self, max_width: usize) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the number of spaces per indentation level ([`Config::tab_spaces`]).
    pub fn with_tab_spaces(mut self, tab_spaces: usize) -> Self {
        self.tab_spaces = tab_spaces;
        self
    }

    /// Returns the maximum width for keeping field-access chains on one line.
    ///
    /// Chains whose estimated length reaches this bound (a fraction of
    /// [`Config::max_width`]) are broken across lines.
    pub fn chain_width(&self) -> usize {
        const CHAIN_WIDTH_RATIO: f32 = 0.6;
        (self.max_width as f32 * CHAIN_WIDTH_RATIO) as usize
    }

    /// Sets the terminator for generated layout ([`Config::line_ending`]).
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

    /// Sets the text wrapping mode ([`Config::wrap_mode`]).
    pub fn with_wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
        self.wrap_mode = wrap_mode;
        self
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

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
}
