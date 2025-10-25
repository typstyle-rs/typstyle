use typst_syntax::{ast::*, SyntaxKind, SyntaxNode};

use super::{
    layout::flow::{FlowItem, FlowStylist},
    prelude::*,
    util::is_comment_node,
    Context, Mode, PrettyPrinter,
};
use crate::ext::{BoolExt, StrExt};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_named(&'a self, ctx: Context, named: Named<'a>) -> ArenaDoc<'a> {
        let mut seen_name = false;
        self.convert_flow_like(ctx, named.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::Colon {
                FlowItem::tight_spaced(self.arena.text(":"))
            } else if let Some(expr) = child.cast() {
                // expr
                FlowItem::spaced_before(self.convert_expr(ctx, expr), seen_name.replace(true))
            } else if let Some(pattern) = child.cast() {
                // pattern
                FlowItem::spaced(self.convert_pattern(ctx, pattern))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_keyed(&'a self, ctx: Context, keyed: Keyed<'a>) -> ArenaDoc<'a> {
        let mut seen_key = false;
        self.convert_flow_like(ctx, keyed.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::Colon {
                FlowItem::tight_spaced(self.arena.text(":"))
            } else if let Some(expr) = child.cast() {
                // key, expr
                FlowItem::spaced_before(self.convert_expr(ctx, expr), seen_key.replace(true))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_spread(&'a self, ctx: Context, spread: Spread<'a>) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, spread.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::Dots {
                FlowItem::spaced_tight(self.arena.text(".."))
            } else if let Some(expr) = child.cast() {
                // expr, sink_ident, sink_expr
                FlowItem::tight_spaced(self.convert_expr(ctx, expr))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_unary(&'a self, ctx: Context, unary: Unary<'a>) -> ArenaDoc<'a> {
        let is_op_keyword = unary.op() == UnOp::Not;
        self.convert_flow_like(ctx, unary.to_untyped(), |ctx, child, _| {
            if UnOp::from_kind(child.kind()).is_some() {
                FlowItem::spaced_tight(self.arena.text(child.text().as_str()))
            } else if let Some(expr) = child.cast() {
                if is_op_keyword {
                    FlowItem::spaced(self.convert_expr(ctx, expr))
                } else {
                    FlowItem::tight_spaced(self.convert_expr(ctx, expr))
                }
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_binary(&'a self, ctx: Context, binary: Binary<'a>) -> ArenaDoc<'a> {
        // Layout every binary expression except assignment as chain.
        if !ctx.break_suppressed && is_chainable_binary(binary) {
            return self
                .parenthesize_if_necessary(ctx, |ctx| self.convert_binary_chain(ctx, binary));
        }
        self.convert_flow_like(ctx, binary.to_untyped(), |ctx, child, _| {
            if BinOp::from_kind(child.kind()).is_some() {
                FlowItem::spaced(self.arena.text(child.text().as_str()))
            } else if let Some(expr) = child.cast() {
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_closure(&'a self, ctx: Context, closure: Closure<'a>) -> ArenaDoc<'a> {
        enum LookAhead {
            Name,
            Params,
            Body,
        }
        let is_named = closure.name().is_some();
        let mut look_ahead = if is_named {
            LookAhead::Name
        } else {
            LookAhead::Params
        };
        self.convert_flow_like(ctx, closure.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::Eq {
                return FlowItem::spaced(self.arena.text("="));
            } else if child.kind() == SyntaxKind::Arrow {
                return FlowItem::spaced(self.arena.text("=>"));
            }
            match look_ahead {
                LookAhead::Name => {
                    if let Some(ident) = child.cast() {
                        look_ahead = LookAhead::Params;
                        return FlowItem::tight(self.convert_ident(ident));
                    }
                }
                LookAhead::Params => {
                    if let Some(params) = child.cast() {
                        look_ahead = LookAhead::Body;
                        return FlowItem::tight_spaced(self.convert_params(ctx, params, !is_named));
                    }
                }
                LookAhead::Body => {
                    if let Some(expr) = child.cast() {
                        let use_braces = if let Expr::Binary(binary) = expr {
                            !is_chainable_binary(binary)
                        } else {
                            true
                        };
                        return FlowItem::spaced(
                            self.convert_expr_with_optional_paren(ctx, expr, use_braces),
                        );
                    }
                }
            }
            FlowItem::none()
        })
    }

    pub(super) fn convert_let_binding(
        &'a self,
        ctx: Context,
        let_binding: LetBinding<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, let_binding.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::Eq {
                FlowItem::spaced(self.arena.text("="))
            } else if let Some(pattern) = child.cast() {
                // Must try pattern before expr
                FlowItem::spaced(self.convert_pattern(ctx, pattern))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_destruct_assignment(
        &'a self,
        ctx: Context,
        destruct_assign: DestructAssignment<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, destruct_assign.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::Eq {
                FlowItem::spaced(self.arena.text("="))
            } else if let Some(pattern) = child.cast() {
                // pattern
                FlowItem::spaced(self.convert_pattern(ctx, pattern))
            } else if let Some(expr) = child.cast() {
                // value
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_contextual(
        &'a self,
        ctx: Context,
        contextual: Contextual<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, contextual.to_untyped())
    }

    pub(super) fn convert_conditional(
        &'a self,
        ctx: Context,
        conditional: Conditional<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, conditional.to_untyped())
    }

    pub(super) fn convert_while_loop(
        &'a self,
        ctx: Context,
        while_loop: WhileLoop<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, while_loop.to_untyped())
    }

    pub(super) fn convert_for_loop(&'a self, ctx: Context, for_loop: ForLoop<'a>) -> ArenaDoc<'a> {
        enum LookAhead {
            Pattern,
            Iterable,
            Body,
        }
        let mut look_ahead = LookAhead::Pattern;
        self.convert_flow_like(ctx, for_loop.to_untyped(), |ctx, child, _| {
            match look_ahead {
                LookAhead::Pattern => {
                    if let Some(pattern) = child.cast() {
                        look_ahead = LookAhead::Iterable;
                        return FlowItem::spaced(self.convert_pattern(ctx, pattern));
                    }
                }
                LookAhead::Iterable => {
                    if let Some(expr) = child.cast() {
                        look_ahead = LookAhead::Body;
                        return FlowItem::spaced(
                            self.convert_expr_with_optional_paren(ctx, expr, false),
                        );
                    }
                }
                LookAhead::Body => {
                    if let Some(expr) = child.cast() {
                        return FlowItem::spaced(self.convert_expr(ctx, expr));
                    }
                }
            }
            FlowItem::none()
        })
    }

    pub(super) fn convert_return(
        &'a self,
        ctx: Context,
        return_stmt: FuncReturn<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, return_stmt.to_untyped())
    }

    pub(super) fn convert_include(
        &'a self,
        ctx: Context,
        include: ModuleInclude<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, include.to_untyped())
    }

    pub(super) fn convert_set_rule(&'a self, ctx: Context, set_rule: SetRule<'a>) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, set_rule.to_untyped(), |ctx, child, _| {
            if let Some(expr) = child.cast() {
                // target or condition
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else if let Some(args) = child.cast() {
                // args
                FlowItem::tight_spaced(self.convert_args(ctx, args, |nodes| {
                    self.convert_parenthesized_args_normal(ctx, args, nodes)
                }))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_show_rule(
        &'a self,
        ctx: Context,
        show_rule: ShowRule<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, show_rule.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::Colon {
                FlowItem::tight_spaced(self.arena.text(":"))
            } else if let Some(expr) = child.cast() {
                // selector or transform
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else {
                FlowItem::none()
            }
        })
    }

    /// Convert a flow-like structure with given item producer.
    pub(super) fn convert_flow_like(
        &'a self,
        ctx: Context,
        node: &'a SyntaxNode,
        producer: impl FnMut(Context, &'a SyntaxNode, &FlowState) -> FlowItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like_iter(ctx, node.children(), producer)
    }

    pub(super) fn convert_flow_like_iter(
        &'a self,
        ctx: Context,
        children: impl Iterator<Item = &'a SyntaxNode>,
        mut producer: impl FnMut(Context, &'a SyntaxNode, &FlowState) -> FlowItem<'a>,
    ) -> ArenaDoc<'a> {
        let mut flow = FlowStylist::new(self);
        let mut peek_line_comment = false;
        let mut peek_hash = false;
        for child in children {
            let state = FlowState {
                at_line_comment: peek_line_comment,
                at_hash: peek_hash,
            };
            peek_line_comment = false;
            peek_hash = false;
            if child.kind().is_keyword()
                && !matches!(child.kind(), SyntaxKind::None | SyntaxKind::Auto)
            {
                flow.push_doc(self.arena.text(child.text().as_str()), true, true);
            } else if is_comment_node(child) {
                if child.kind() == SyntaxKind::LineComment {
                    peek_line_comment = true; // defers the linebreak
                }
                flow.push_comment(
                    self.convert_comment(ctx, child),
                    child.kind() == SyntaxKind::BlockComment,
                );
            } else if child.kind() == SyntaxKind::Hash {
                flow.push_doc(self.arena.text("#"), true, false);
                peek_hash = true;
            } else {
                let ctx = ctx.with_mode_if(Mode::Code, state.at_hash);
                let item = producer(ctx, child, &state); // the producer should know this linebreak
                if state.at_line_comment
                    && child.kind() == SyntaxKind::Space
                    && child.text().has_linebreak()
                {
                    flow.push_doc(self.arena.hard_line(), false, false);
                    flow.enter_new_line();
                } else if let Some(repr) = item.0 {
                    flow.push_doc(repr.doc, repr.space_before, repr.space_after);
                }
            }
        }
        flow.into_doc()
    }

    /// Convert nodes with only keywords, exprs (followed by space), and comments.
    pub(super) fn convert_expr_flow(&'a self, ctx: Context, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, node, |ctx, child, _| {
            if let Some(expr) = child.cast() {
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else {
                FlowItem::none()
            }
        })
    }
}

pub(super) struct FlowState {
    /// The previous item was a line comment.
    pub at_line_comment: bool,
    /// The previous item was a hash.
    pub at_hash: bool,
}

/// Returns whether a binary expression is chainable.
///
/// A binary expression is considered chainable if its operator precedence is
/// higher than that of the assignment operator. This is used to determine if the
/// expression should be laid out as a chain of binary operations.
fn is_chainable_binary(binary: Binary<'_>) -> bool {
    binary.op().precedence() > BinOp::Assign.precedence()
}
