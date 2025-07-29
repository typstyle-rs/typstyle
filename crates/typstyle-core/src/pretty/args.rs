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

/// Split the arguments of a function call into parenthesized and trailing nodes.
pub fn split_paren_args(args: Args) -> (&[SyntaxNode], &[SyntaxNode]) {
    let children = args.to_untyped().children().as_slice();
    // split args, find first `)` and split there (inclusive)
    if let Some(idx) = children
        .iter()
        .position(|n| n.kind() == SyntaxKind::RightParen)
    {
        children.split_at(idx + 1)
    } else {
        // no parens at all → all nodes are "trailing"
        (&[][..], children)
    }
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
        Expr::Raw(raw) => raw.block(),
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
