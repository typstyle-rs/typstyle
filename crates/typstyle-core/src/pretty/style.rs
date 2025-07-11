use typst_syntax::{ast::*, SyntaxKind, SyntaxNode};

use crate::ext::StrExt;

/// A style for formatting items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldStyle {
    /// Fold items if them can fit in a single line
    Fit,
    /// Never fold items
    Never,
    /// Always fold items
    Always,
    /// Try to fold items except the last one in a single line
    Compact,
}

/// A syntax node is multiline flavored, if a line break appears before the first item.
pub fn is_multiline_flavored(node: &SyntaxNode) -> bool {
    for child in node.children() {
        if child.kind() == SyntaxKind::Space {
            return child.text().has_linebreak();
        }
        // To cover most cases, we use `len` to skip trivias (e.g., commas)
        if child.is::<Expr>() || child.children().len() > 0 {
            break;
        }
    }
    false
}
