//! The context threaded through conversion: the syntactic [`Mode`], whether line breaks
//! are suppressed, and how math may align.

/// The state carried through a conversion pass.
///
/// It is copy-on-write: every `with_*` method returns an adjusted copy, so enclosing
/// scopes are never affected by a change made deeper in the tree.
#[derive(Default, Clone, Copy)]
pub struct Context {
    /// The syntactic context the node appears in, which decides how line breaks and
    /// spaces around it may be laid out.
    pub mode: Mode,
    /// Whether line breaks are forbidden in the current subtree, e.g., inside a heading
    /// or a table cell.
    pub break_suppressed: bool,
    /// Whether and how the math in the current subtree may be laid out as an aligned
    /// grid.
    pub align_mode: AlignMode,
}

impl Context {
    /// Enters the given [`Mode`].
    ///
    /// Entering [`Mode::CodeCont`] from [`Mode::Math`] is a no-op: math is already
    /// free-form, and downgrading it would change how spaces break.
    pub fn with_mode(self, mode: Mode) -> Self {
        // We should never enter CodeCont mode when in math.
        if matches!((self.mode, mode), (Mode::Math, Mode::CodeCont)) {
            self
        } else {
            Self { mode, ..self }
        }
    }

    /// Enters the given [`Mode`] only if `cond` holds; otherwise keeps the current one.
    pub fn with_mode_if(self, mode: Mode, cond: bool) -> Self {
        Self {
            mode: if cond { mode } else { self.mode },
            ..self
        }
    }

    /// Forbids line breaks in the resulting subtree.
    pub fn suppress_breaks(self) -> Self {
        Self {
            break_suppressed: true,
            ..self
        }
    }

    /// Records how math in the subtree may align; see [`AlignMode`] for the precedence
    /// between the current and the new mode.
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

/// The syntactic context a node appears in.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Inside prose markup, where spaces and line breaks are significant.
    #[default]
    Markup,
    /// Inside a code block or expression, where line breaks end the expression.
    Code,
    /// A continued code mode, where the code can freely span multiple lines.
    /// Corresponds to `typst_syntax::parser::AtNewline::Continue`.
    CodeCont,
    /// Inside math, where content is free-form and breaks follow the source.
    Math,
}

impl Mode {
    /// Returns `true` if the current mode is markup.
    pub fn is_markup(self) -> bool {
        self == Self::Markup
    }

    /// Returns `true` if the current mode is either `Code` or `CodeCont`.
    pub fn is_code(self) -> bool {
        self == Self::Code || self == Self::CodeCont
    }

    /// Returns `true` if the current mode is [`Mode::CodeCont`], i.e., line breaks do
    /// not terminate the expression.
    pub fn is_code_continued(self) -> bool {
        self == Self::CodeCont
    }

    /// Returns `true` if the current mode is math.
    pub fn is_math(self) -> bool {
        self == Self::Math
    }
}

/// Whether and at which nesting level math may be laid out as an aligned grid.
///
/// [`Self::Never`] and [`Self::Inner`] are sticky: once set, later [`Context::aligned`]
/// calls cannot override them. [`Self::Auto`] and [`Self::Outer`] are replaced by the
/// next [`Context::aligned`] call.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AlignMode {
    /// Alignment not decided yet; the next [`Context::aligned`] call decides it.
    #[default]
    Auto,
    /// The current scope forms the outermost aligned grid.
    Outer,
    /// A nested scope whose cells align within the enclosing grid.
    Inner,
    /// Grid alignment is disabled for this scope and all descendants.
    Never,
}
