//! Conversion of operators, bindings, closures, control flow, and set/show rules.

use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{
    Context, Mode, PrettyPrinter,
    layout::flow::{FlowItem, FlowStylist},
    prelude::*,
};
use crate::ext::{BoolExt, StrExt};

impl<'a> PrettyPrinter<'a> {
    /// Converts a named pair, `name: value`, as found in dicts, arguments, and patterns.
    pub(super) fn convert_named(&'a self, ctx: Context, named: Named<'a>) -> ArenaDoc<'a> {
        let mut seen_name = false;
        self.convert_flow_like(ctx, named.to_untyped(), |ctx, child| {
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

    /// Converts a keyed dictionary entry, `key: value`, where the key is an expression.
    pub(super) fn convert_keyed(&'a self, ctx: Context, keyed: Keyed<'a>) -> ArenaDoc<'a> {
        let mut seen_key = false;
        self.convert_flow_like(ctx, keyed.to_untyped(), |ctx, child| {
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

    /// Converts a spread, `..expr`.
    pub(super) fn convert_spread(&'a self, ctx: Context, spread: Spread<'a>) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, spread.to_untyped(), |ctx, child| {
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

    /// Converts a unary expression.
    ///
    /// Only the keyword operator `not` takes a space before its operand.
    pub(super) fn convert_unary(&'a self, ctx: Context, unary: Unary<'a>) -> ArenaDoc<'a> {
        let is_op_keyword = unary.op() == UnOp::Not;
        self.convert_flow_like(ctx, unary.to_untyped(), |ctx, child| {
            if UnOp::from_kind(child.kind()).is_some() {
                FlowItem::spaced_tight(self.arena.text(child.leaf_text().as_str()))
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

    /// Converts a binary expression.
    ///
    /// Chainable operators are laid out as a chain via [`Self::convert_binary_chain`];
    /// assignments and break-suppressed contexts use a plain flow.
    pub(super) fn convert_binary(&'a self, ctx: Context, binary: Binary<'a>) -> ArenaDoc<'a> {
        // Layout every binary expression except assignment as chain.
        if !ctx.break_suppressed && is_chainable_binary(binary) {
            return self
                .parenthesize_if_necessary(ctx, |ctx| self.convert_binary_chain(ctx, binary));
        }
        self.convert_flow_like(ctx, binary.to_untyped(), |ctx, child| {
            if BinOp::from_kind(child.kind()).is_some() {
                FlowItem::spaced(self.arena.text(child.leaf_text().as_str()))
            } else if let Some(expr) = child.cast() {
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else {
                FlowItem::none()
            }
        })
    }

    /// Converts a closure, such as `f(x) = body` or `x => body`.
    pub(super) fn convert_closure(&'a self, ctx: Context, closure: Closure<'a>) -> ArenaDoc<'a> {
        /// The next part of the closure to expect.
        enum LookAhead {
            /// The optional function name.
            Name,
            /// The parameter list.
            Params,
            /// The body expression.
            Body,
        }
        let is_named = closure.name().is_some();
        let mut look_ahead = if is_named {
            LookAhead::Name
        } else {
            LookAhead::Params
        };
        self.convert_flow_like(ctx, closure.to_untyped(), |ctx, child| {
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

    /// Converts a `let` binding, with or without an initial value.
    pub(super) fn convert_let_binding(
        &'a self,
        ctx: Context,
        let_binding: LetBinding<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, let_binding.to_untyped(), |ctx, child| {
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

    /// Converts a destructuring assignment, `pattern = value`.
    pub(super) fn convert_destruct_assignment(
        &'a self,
        ctx: Context,
        destruct_assign: DestructAssignment<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, destruct_assign.to_untyped(), |ctx, child| {
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

    /// Converts a `context` expression.
    pub(super) fn convert_contextual(
        &'a self,
        ctx: Context,
        contextual: Contextual<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, contextual.to_untyped())
    }

    /// Converts an `if`-`else` conditional.
    pub(super) fn convert_conditional(
        &'a self,
        ctx: Context,
        conditional: Conditional<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, conditional.to_untyped())
    }

    /// Converts a `while` loop.
    pub(super) fn convert_while_loop(
        &'a self,
        ctx: Context,
        while_loop: WhileLoop<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, while_loop.to_untyped())
    }

    /// Converts a `for` loop.
    pub(super) fn convert_for_loop(&'a self, ctx: Context, for_loop: ForLoop<'a>) -> ArenaDoc<'a> {
        /// The next part of the loop header to expect.
        enum LookAhead {
            /// The iterated pattern.
            Pattern,
            /// The iterable expression.
            Iterable,
            /// The body expression.
            Body,
        }
        let mut look_ahead = LookAhead::Pattern;
        self.convert_flow_like(ctx, for_loop.to_untyped(), |ctx, child| {
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

    /// Converts a `return` statement, with or without a value.
    pub(super) fn convert_return(
        &'a self,
        ctx: Context,
        return_stmt: FuncReturn<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, return_stmt.to_untyped())
    }

    /// Converts a module include.
    pub(super) fn convert_include(
        &'a self,
        ctx: Context,
        include: ModuleInclude<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr_flow(ctx, include.to_untyped())
    }

    /// Converts a `set` rule.
    pub(super) fn convert_set_rule(&'a self, ctx: Context, set_rule: SetRule<'a>) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, set_rule.to_untyped(), |ctx, child| {
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

    /// Converts a `show` rule, `selector: transform`.
    pub(super) fn convert_show_rule(
        &'a self,
        ctx: Context,
        show_rule: ShowRule<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, show_rule.to_untyped(), |ctx, child| {
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
        producer: impl FnMut(Context, &'a SyntaxNode) -> FlowItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like_iter(ctx, node.children(), producer)
    }

    /// Converts an iterator of nodes as a flow with the given item producer.
    ///
    /// Keywords, comments, and `#` are handled here; everything else is delegated to
    /// `producer`, which receives the effective context of the current child. The
    /// line break after a line comment is deferred to the next non-comment child.
    /// Children must retain source order and intervening trivia so attached code
    /// terminators can be distinguished from math row separators.
    pub(super) fn convert_flow_like_iter(
        &'a self,
        ctx: Context,
        children: impl IntoIterator<Item = &'a SyntaxNode>,
        mut producer: impl FnMut(Context, &'a SyntaxNode) -> FlowItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like_iter_impl(ctx, children.into_iter(), &mut producer)
    }

    // Using dyn trait object for producer to avoid monomorphization bloat.
    fn convert_flow_like_iter_impl(
        &'a self,
        ctx: Context,
        children: impl Iterator<Item = &'a SyntaxNode>,
        producer: &mut dyn FnMut(Context, &'a SyntaxNode) -> FlowItem<'a>,
    ) -> ArenaDoc<'a> {
        let mut flow = FlowStylist::new(self);
        let mut children = children.peekable();
        while let Some(child) = children.next() {
            match child.kind() {
                kind if kind.is_keyword()
                    && !matches!(kind, SyntaxKind::None | SyntaxKind::Auto) =>
                {
                    flow.push_doc(self.arena.text(child.leaf_text().as_str()), true, true);
                }
                SyntaxKind::LineComment => {
                    flow.push_comment(self.convert_comment(ctx, child), false);
                    // Consume next linebreak if present
                    if let Some(linebreak) = children.next_if(|next| {
                        next.kind() == SyntaxKind::Space && next.leaf_text().has_linebreak()
                    }) {
                        let _ = producer(ctx, linebreak).0; // Let the producer know about the linebreak
                        flow.push_doc(self.arena.hard_line(), false, false);
                        flow.enter_new_line();
                    }
                }
                SyntaxKind::BlockComment => {
                    flow.push_comment(self.convert_comment(ctx, child), true);
                }
                SyntaxKind::Hash => {
                    flow.push_doc(self.arena.text("#"), true, false);
                    // Must follow with a code expression
                    let child_ctx = ctx.with_mode(Mode::Code);
                    let Some(child) = children.next() else {
                        break; // Unreachable when syntax is valid
                    };
                    let item = producer(child_ctx, child);
                    let Some(repr) = item.0 else {
                        continue;
                    };
                    flow.push_doc(repr.doc, repr.space_before, repr.space_after);
                    if children
                        .next_if(|next| next.kind() == SyntaxKind::Semicolon)
                        .is_some()
                    {
                        flow.push_doc(self.arena.text(";"), false, false);
                    }
                }
                _ => {
                    if let Some(repr) = producer(ctx, child).0 {
                        flow.push_doc(repr.doc, repr.space_before, repr.space_after);
                    }
                }
            }
        }
        flow.into_doc()
    }

    /// Convert nodes with only keywords, exprs (followed by space), and comments.
    pub(super) fn convert_expr_flow(&'a self, ctx: Context, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, node, |ctx, child| {
            if let Some(expr) = child.cast() {
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else {
                FlowItem::none()
            }
        })
    }
}

/// Returns whether a binary expression is chainable.
///
/// A binary expression is considered chainable if its operator precedence is
/// higher than that of the assignment operator. This is used to determine if the
/// expression should be laid out as a chain of binary operations.
fn is_chainable_binary(binary: Binary<'_>) -> bool {
    binary.op().precedence() > BinOp::Assign.precedence()
}
