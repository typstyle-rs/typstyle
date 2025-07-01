use typst_syntax::{ast::*, SyntaxKind, SyntaxNode};

pub fn is_only_one_and<T>(
    mut iterator: impl Iterator<Item = T>,
    f: impl FnOnce(&T) -> bool,
) -> bool {
    iterator
        .next()
        .is_some_and(|first| iterator.next().is_none() && f(&first))
}

pub fn is_empty_or_one_if<T>(
    mut iterator: impl Iterator<Item = T>,
    f: impl FnOnce(&T) -> bool,
) -> bool {
    iterator
        .next()
        .is_none_or(|it| iterator.next().is_none() && f(&it))
}

pub fn is_comment_node(node: &SyntaxNode) -> bool {
    matches!(
        node.kind(),
        SyntaxKind::LineComment | SyntaxKind::BlockComment
    )
}

pub fn has_comment_children(node: &SyntaxNode) -> bool {
    node.children().any(is_comment_node)
}

pub(super) fn func_name(node: FuncCall<'_>) -> Option<&str> {
    match node.callee() {
        Expr::Ident(ident) => Some(ident.as_str()),
        Expr::FieldAccess(field_access) => Some(field_access.field().as_str()),
        _ => None,
    }
}

/// Like `f()`, `f(x, y)`, not `f[]`
pub(super) fn has_parenthesized_args(node: Args<'_>) -> bool {
    node.to_untyped()
        .children()
        .next()
        .is_some_and(|child| child.kind() == SyntaxKind::LeftParen)
}

pub(super) fn get_parenthesized_args_untyped(node: Args<'_>) -> impl Iterator<Item = &SyntaxNode> {
    node.to_untyped()
        .children()
        .skip_while(|node| node.kind() != SyntaxKind::LeftParen)
        .take_while(|node| node.kind() != SyntaxKind::RightParen)
}

pub(super) fn get_parenthesized_args(node: Args<'_>) -> impl Iterator<Item = Arg<'_>> {
    get_parenthesized_args_untyped(node).filter_map(|node| node.cast::<Arg>())
}
