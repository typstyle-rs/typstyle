use typst_syntax::ast::*;

use super::{prelude::*, util::has_comment_children, Context, Mode};
use crate::PrettyPrinter;

impl<'a> PrettyPrinter<'a> {
    /// We do not care whether it is `Pattern` or `Expr`.
    /// It is safe to treat it as `Pattern`, since `Pattern` can be `Expr`.
    pub(super) fn convert_parenthesized(
        &'a self,
        ctx: Context,
        parenthesized: Parenthesized<'a>,
    ) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        if let Pattern::Parenthesized(paren) = parenthesized.pattern() {
            if !has_comment_children(parenthesized.to_untyped()) {
                // Remove a layer of paren if no comment inside.
                return self.convert_parenthesized(ctx, paren);
            }
        }

        // Treat is as a list with a single item.
        self.convert_parenthesized_impl(ctx, parenthesized)
    }

    /// Convert an expression with optional parentheses.
    /// If the expression is a parenthesized expression, a code block, a content block, or a function call,
    /// the expression will be converted without parentheses.
    /// Otherwise, the expression will be converted with parentheses if it is laid out on multiple lines.
    pub(super) fn convert_expr_with_optional_paren(
        &'a self,
        ctx: Context,
        expr: Expr<'a>,
        use_braces: bool,
    ) -> ArenaDoc<'a> {
        if ctx.break_suppressed || !is_paren_needed(expr) {
            return self.convert_expr(ctx, expr);
        }
        let (mode, delims) = if use_braces {
            (Mode::Code, ("{", "}"))
        } else {
            (Mode::CodeCont, ("(", ")"))
        };
        let ctx = ctx.with_mode(mode);
        self.optional_paren(&self.arena, self.convert_expr(ctx, expr), delims)
    }

    /// Parenthesize the body if necessary.
    ///
    /// We must enter continued-code mode before evaluating body.
    pub(super) fn parenthesize_if_necessary(
        &'a self,
        ctx: Context,
        body: impl FnOnce(Context) -> ArenaDoc<'a>,
    ) -> ArenaDoc<'a> {
        if ctx.mode.is_code_continued() {
            return body(ctx);
        }
        // SAFETY:
        // - If without paren, the entire expression is in one line, thus safe.
        // - If with paren, surely safe.
        let ctx = ctx.with_mode(Mode::CodeCont);
        self.optional_paren(&self.arena, body(ctx), ("(", ")"))
    }

    /// Wrap the body with parentheses if the body is layouted on multiple lines.
    fn optional_paren(
        &'a self,
        arena: &'a Arena<'a>,
        body: ArenaDoc<'a>,
        delims: (&'static str, &'static str),
    ) -> ArenaDoc<'a> {
        let open = arena.text(delims.0).when_group_break();
        let close = arena.text(delims.1).when_group_break();
        (open + self.block_indent(body) + close).group()
    }
}

/// Checks if parentheses are needed for an expression that may span multiple lines.
fn is_paren_needed(expr: Expr<'_>) -> bool {
    !matches!(
        expr,
        Expr::Parenthesized(_)
            | Expr::Code(_)
            | Expr::Content(_)
            | Expr::FuncCall(_)
            | Expr::Array(_)
            | Expr::Dict(_)
            | Expr::Conditional(_)
            | Expr::While(_)
            | Expr::For(_)
            | Expr::Contextual(_)
            | Expr::Closure(_)
            | Expr::Raw(_)
    )
}
