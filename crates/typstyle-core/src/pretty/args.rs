//! Helpers for inspecting function-call arguments, used to decide how they are laid
//! out.

use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use crate::{
    AttrStore,
    pretty::{
        code_chain::resolve_dot_chain, style::is_multiline_flavored, table,
        util::has_comment_children,
    },
};

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
pub fn split_paren_args(args: Args<'_>) -> (&[SyntaxNode], &[SyntaxNode]) {
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
        Expr::CodeBlock(_)
            | Expr::Conditional(_)
            | Expr::WhileLoop(_)
            | Expr::ForLoop(_)
            | Expr::Contextual(_)
            | Expr::Closure(_)
    )
}

/// Identify expressions that can be combined with preceding arguments in compact layout.
pub fn is_combinable(expr: Expr, attr_store: &AttrStore) -> bool {
    match expr {
        Expr::Raw(raw) => raw.block(),
        Expr::ContentBlock(content) => content.body().exprs().next().is_some(),
        Expr::Array(array) => array.items().next().is_some(),
        Expr::Dict(dict) => dict.items().next().is_some(),
        Expr::FuncCall(func_call) => is_combinable_func_call(func_call, attr_store),
        _ => is_blocky(expr),
    }
}

/// Returns whether a function call is structured enough to combine with preceding
/// arguments in compact layout without awkwardly splitting a simple single-line call.
fn is_combinable_func_call(func_call: FuncCall, attr_store: &AttrStore) -> bool {
    if resolve_dot_chain(func_call.to_untyped()).skip(1).any(|it| {
        it.cast::<FuncCall>()
            .is_some_and(|fc| fc.args().items().next().is_some())
    }) {
        return false;
    }

    let (paren_nodes, trailing_nodes) = split_paren_args(func_call.args());

    if trailing_nodes
        .iter()
        .filter_map(SyntaxNode::cast::<ContentBlock>)
        .any(|cb| is_multiline_content_block(cb, attr_store))
    {
        return true;
    }

    let mut paren_args = paren_nodes.iter().filter_map(SyntaxNode::cast::<Arg>);
    let Some(first_arg) = paren_args.next() else {
        return false;
    };

    if table::is_table(func_call)
        && paren_args.clone().next().is_some()
        && std::iter::once(first_arg)
            .chain(paren_args.clone())
            .any(|arg| matches!(arg, Arg::Pos(_)))
    {
        return true;
    }

    let args_node = func_call.args().to_untyped();
    if is_multiline_flavored(args_node)
        || paren_nodes
            .iter()
            .any(|child| child.kind() == SyntaxKind::LineComment)
    {
        return true;
    }

    std::iter::once(first_arg)
        .chain(paren_args)
        .any(|arg| is_multiline_arg(unwrap_expr_deep(arg), attr_store))
}

fn has_line_comment_children(node: &SyntaxNode) -> bool {
    node.children()
        .any(|child| child.kind() == SyntaxKind::LineComment)
}

fn is_multiline_content_block(content: ContentBlock, attr_store: &AttrStore) -> bool {
    let node = content.to_untyped();
    attr_store.is_multiline(node) || attr_store.has_multiline_str(node)
}

fn is_multiline_arg(expr: Expr, attr_store: &AttrStore) -> bool {
    match expr {
        Expr::Parenthesized(paren) => is_multiline_arg(paren.expr(), attr_store),
        Expr::Raw(raw) => raw.block() || raw.lines().nth(1).is_some(),
        Expr::Str(_) => attr_store.has_multiline_str(expr.to_untyped()),
        Expr::Equation(eq) => attr_store.is_multiline(eq.to_untyped()),
        Expr::ContentBlock(content) => is_multiline_content_block(content, attr_store),
        Expr::CodeBlock(block) => {
            let body_node = block.body().to_untyped();
            let has_comment = has_comment_children(body_node);
            let mut exprs = block.body().exprs();
            if exprs.next().is_none() && !has_comment {
                return false;
            }
            exprs.next().is_some() || attr_store.is_multiline(block.to_untyped()) || has_comment
        }
        Expr::Array(array) => {
            let node = array.to_untyped();
            is_multiline_flavored(node)
                || has_line_comment_children(node)
                || ((attr_store.is_multiline(node) || attr_store.has_multiline_str(node))
                    && array.items().any(|item| match item {
                        ArrayItem::Pos(e) => is_multiline_arg(e, attr_store),
                        ArrayItem::Spread(s) => is_multiline_arg(s.expr(), attr_store),
                    }))
        }
        Expr::Dict(dict) => {
            let node = dict.to_untyped();
            is_multiline_flavored(node)
                || has_line_comment_children(node)
                || ((attr_store.is_multiline(node) || attr_store.has_multiline_str(node))
                    && dict.items().any(|item| match item {
                        DictItem::Named(n) => is_multiline_arg(n.expr(), attr_store),
                        DictItem::Keyed(k) => {
                            is_multiline_arg(k.key(), attr_store)
                                || is_multiline_arg(k.expr(), attr_store)
                        }
                        DictItem::Spread(s) => is_multiline_arg(s.expr(), attr_store),
                    }))
        }
        Expr::Closure(closure) => {
            let params = closure.params().to_untyped();
            is_multiline_flavored(params)
                || has_line_comment_children(params)
                || is_multiline_arg(closure.body(), attr_store)
        }
        Expr::Contextual(contextual) => is_multiline_arg(contextual.body(), attr_store),
        Expr::Conditional(cond) => {
            attr_store.is_multiline(cond.to_untyped())
                || is_multiline_arg(cond.if_body(), attr_store)
                || cond
                    .else_body()
                    .is_some_and(|else_body| is_multiline_arg(else_body, attr_store))
        }
        Expr::WhileLoop(while_loop) => {
            attr_store.is_multiline(while_loop.to_untyped())
                || is_multiline_arg(while_loop.body(), attr_store)
        }
        Expr::ForLoop(for_loop) => {
            attr_store.is_multiline(for_loop.to_untyped())
                || is_multiline_arg(for_loop.body(), attr_store)
        }
        Expr::FuncCall(func_call) => {
            let node = func_call.to_untyped();
            (attr_store.is_multiline(node)
                || attr_store.has_multiline_str(node)
                || table::is_table(func_call))
                && is_combinable_func_call(func_call, attr_store)
        }
        _ => false,
    }
}
