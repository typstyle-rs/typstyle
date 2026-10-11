//! Small predicates and helpers over syntax nodes shared by the converters and
//! stylists.

use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

/// Returns `true` if the iterator yields exactly one item and it satisfies `f`.
pub fn is_only_one_and<T>(
    mut iterator: impl Iterator<Item = T>,
    f: impl FnOnce(&T) -> bool,
) -> bool {
    iterator
        .next()
        .is_some_and(|first| iterator.next().is_none() && f(&first))
}

/// Returns `true` if the iterator is empty, or yields exactly one item that satisfies
/// `f`.
pub fn is_empty_or_one_if<T>(
    mut iterator: impl Iterator<Item = T>,
    f: impl FnOnce(&T) -> bool,
) -> bool {
    iterator
        .next()
        .is_none_or(|it| iterator.next().is_none() && f(&it))
}

/// Returns whether the node is a line or block comment.
pub fn is_comment_node(node: &SyntaxNode) -> bool {
    matches!(
        node.kind(),
        SyntaxKind::LineComment | SyntaxKind::BlockComment
    )
}

/// Returns whether any direct child of the node is a comment.
pub fn has_comment_children(node: &SyntaxNode) -> bool {
    node.children().any(is_comment_node)
}

/// Returns the name of the called function, either an identifier or a field name.
pub(super) fn func_name(node: FuncCall<'_>) -> Option<&str> {
    match node.callee() {
        Expr::Ident(ident) => Some(ident.as_str()),
        Expr::FieldAccess(field_access) => Some(field_access.field().as_str()),
        _ => None,
    }
}
