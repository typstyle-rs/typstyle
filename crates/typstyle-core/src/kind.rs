//! Classification of the syntax kinds the formatter handles as a unit.
//!
//! [`SyntaxKind`] is fine-grained; the formatter repeatedly tests whole sets of
//! kinds. Each set is defined exactly once here so those tests cannot drift apart.

use typst_syntax::SyntaxKind;

/// Predicates over [`SyntaxKind`] for the sets the formatter handles uniformly.
pub(crate) trait SyntaxKindExt: Copy {
    /// Whether this kind is a line or block comment.
    fn is_comment(self) -> bool;

    /// Whether this kind is whitespace or a comment inside a line of markup.
    ///
    /// Unlike [`Self::is_trivia`], this excludes `Parbreak` and `Shebang`.
    fn is_space_or_comment(self) -> bool;

    /// Whether this kind is markup that must start its own line: `Heading`, `ListItem`,
    /// `EnumItem`, or `TermItem`.
    fn is_block_markup(self) -> bool;

    /// Whether this kind is a list-like markup item: `ListItem`, `EnumItem`, or `TermItem`.
    fn is_list_item_markup(self) -> bool;

    /// Whether this kind contains math and can carry alignment points: `Math` or
    /// `MathDelimited`.
    fn is_math_container(self) -> bool;
}

impl SyntaxKindExt for SyntaxKind {
    fn is_comment(self) -> bool {
        matches!(self, Self::LineComment | Self::BlockComment)
    }

    fn is_space_or_comment(self) -> bool {
        matches!(self, Self::Space | Self::LineComment | Self::BlockComment)
    }

    fn is_block_markup(self) -> bool {
        matches!(
            self,
            Self::Heading | Self::ListItem | Self::EnumItem | Self::TermItem
        )
    }

    fn is_list_item_markup(self) -> bool {
        matches!(self, Self::ListItem | Self::EnumItem | Self::TermItem)
    }

    fn is_math_container(self) -> bool {
        matches!(self, Self::Math | Self::MathDelimited)
    }
}
