#[derive(Default, Clone, Copy)]
pub struct Context {
    pub mode: Mode,
    pub break_suppressed: bool,
    pub align_mode: AlignMode,
}

impl Context {
    pub fn with_mode(self, mode: Mode) -> Self {
        // We should never enter CodeCont mode when in math.
        if matches!((self.mode, mode), (Mode::Math, Mode::CodeCont)) {
            self
        } else {
            Self { mode, ..self }
        }
    }

    pub fn with_mode_if(self, mode: Mode, cond: bool) -> Self {
        Self {
            mode: if cond { mode } else { self.mode },
            ..self
        }
    }

    pub fn suppress_breaks(self) -> Self {
        Self {
            break_suppressed: true,
            ..self
        }
    }

    pub fn aligned(self, mode: AlignMode) -> Self {
        Self {
            align_mode: match (self.align_mode, mode) {
                (_, AlignMode::Never) | (AlignMode::Never, _) => AlignMode::Never,
                (AlignMode::Inner, _) => AlignMode::Inner,
                (AlignMode::Outer, _) => mode,
                (AlignMode::Auto, _) => mode,
            },
            ..self
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Markup,
    Code,
    /// A continued code mode, where the code can freely span multiple lines.
    /// Corresponds to `typst_syntax::parser::AtNewline::Continue`.
    CodeCont,
    Math,
}

impl Mode {
    pub fn is_markup(self) -> bool {
        self == Self::Markup
    }

    /// Returns `true` if the current mode is either `Code` or `CodeCont`.
    pub fn is_code(self) -> bool {
        self == Self::Code || self == Self::CodeCont
    }

    pub fn is_code_continued(self) -> bool {
        self == Self::CodeCont
    }

    pub fn is_math(self) -> bool {
        self == Self::Math
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AlignMode {
    #[default]
    Auto,
    Outer,
    Inner,
    Never,
}
