use typst_syntax::{ast::*, SyntaxKind, SyntaxNode};

use crate::pretty::{code_chain::resolve_dot_chain, util::is_empty_or_one_if};

/// Get the inner expression of an argument.
pub fn unwrap_expr(arg: Arg) -> Expr {
    match arg {
        Arg::Pos(p) => p,
        Arg::Named(n) => n.expr(),
        Arg::Spread(s) => s.expr(),
    }
}

/// Get the inner expression of an argument, with parentheses removed.
pub fn unwrap_expr_deep(arg: Arg) -> Expr {
    let mut expr = unwrap_expr(arg);
    while let Expr::Parenthesized(inner) = expr {
        expr = inner.expr();
    }
    expr
}

/// Like `f()`, `f(x, y)`, not `f[]`
pub fn has_parenthesized_args(node: Args<'_>) -> bool {
    node.to_untyped()
        .children()
        .next()
        .is_some_and(|child| child.kind() == SyntaxKind::LeftParen)
}

pub fn get_parenthesized_args_untyped(node: Args<'_>) -> impl Iterator<Item = &SyntaxNode> {
    node.to_untyped()
        .children()
        .skip_while(|node| node.kind() != SyntaxKind::LeftParen)
        .take_while(|node| node.kind() != SyntaxKind::RightParen)
}

pub fn get_parenthesized_args(node: Args<'_>) -> impl Iterator<Item = Arg<'_>> {
    get_parenthesized_args_untyped(node).filter_map(|node| node.cast::<Arg>())
}

/// Identify block‐like expressions that deserve their own lines.
pub fn is_blocky(expr: Expr) -> bool {
    matches!(
        expr,
        Expr::Code(_)
            | Expr::Conditional(_)
            | Expr::While(_)
            | Expr::For(_)
            | Expr::Contextual(_)
            | Expr::Closure(_)
    )
}

/// Identify simple expressions we can “smoosh” on one line.
pub fn is_combinable(expr: Expr) -> bool {
    match expr {
        Expr::Content(content) => content.body().exprs().next().is_some(),
        Expr::Array(array) => array.items().next().is_some(),
        Expr::Dict(dict) => dict.items().next().is_some(),
        Expr::FuncCall(func_call) => {
            !is_empty_or_one_if(func_call.args().items(), |&arg| {
                is_literal_or_ident(unwrap_expr(arg))
            }) && !resolve_dot_chain(func_call.to_untyped()).skip(1).any(|it| {
                it.cast::<FuncCall>()
                    .is_some_and(|fc| fc.args().items().next().is_some())
            })
        }
        _ => is_blocky(expr),
    }
}

fn is_literal_or_ident(expr: Expr) -> bool {
    expr.is_literal() || matches!(expr, Expr::Ident(_))
}
