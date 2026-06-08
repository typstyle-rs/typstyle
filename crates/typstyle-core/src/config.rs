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
    /// Text wrapping mode for markup.
    pub wrap_mode: WrapMode,
}

/// Text wrapping mode for markup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WrapMode {
    /// Do not wrap text.
    #[default]
    None,
    /// Wrap text to fit within the line width.
    Fill,
    /// Place each sentence on its own line.
    SentencePerLine,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_spaces: 2,
            max_width: 80,
            blank_lines_upper_bound: 1,
            reorder_import_items: true,
            collapse_markup_spaces: false,
            wrap_mode: WrapMode::None,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_width(mut self, max_width: usize) -> Self {
        self.max_width = max_width;
        self
    }

    pub fn with_tab_spaces(mut self, tab_spaces: usize) -> Self {
        self.tab_spaces = tab_spaces;
        self
    }

    pub fn chain_width(&self) -> usize {
        const CHAIN_WIDTH_RATIO: f32 = 0.6;
        (self.max_width as f32 * CHAIN_WIDTH_RATIO) as usize
    }

    pub fn with_wrap_text(mut self, wrap_text: bool) -> Self {
        self.wrap_mode = if wrap_text {
            WrapMode::Fill
        } else {
            WrapMode::None
        };
        self
    }

    pub fn with_wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
        self.wrap_mode = wrap_mode;
        self
    }
}
