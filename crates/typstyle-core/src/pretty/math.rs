use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{
    Context, Mode, PrettyPrinter,
    context::AlignMode,
    layout::{
        flow::FlowItem,
        list::{ListStyle, ListStylist},
    },
    prelude::*,
    style::{FoldStyle, is_multiline_flavored},
    util::is_comment_node,
};
use crate::ext::StrExt;

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_equation(&'a self, ctx: Context, equation: Equation<'a>) -> ArenaDoc<'a> {
        fn has_trailing_linebreak(equation: Equation) -> bool {
            (equation.to_untyped().children().nth_back(1))
                .is_some_and(|it| it.kind() == SyntaxKind::Space)
                && (equation.to_untyped().children().nth_back(2)).is_some_and(|it| {
                    it.cast::<Math>().is_some_and(|math| {
                        (math.to_untyped().children().last())
                            .is_some_and(|it| it.kind() == SyntaxKind::Linebreak)
                    })
                })
        }

        let ctx = ctx.with_mode(Mode::Math);

        let is_block = equation.block();

        let convert_math_padded = |ctx: Context, child: &'a SyntaxNode| {
            let math = child.cast::<Math>()?;
            if math.to_untyped().children().len() == 0 {
                return Option::None;
            }
            let ctx = if is_block {
                ctx
            } else {
                ctx.aligned(AlignMode::Never)
            };
            let body = self.convert_math(ctx, math);
            let body = if !is_block && has_trailing_linebreak(equation) {
                body + self.arena.space()
            } else {
                body
            };
            Some(body)
        };

        let fold_style = if !is_block || ctx.break_suppressed {
            FoldStyle::Always
        } else if is_multiline_flavored(equation.to_untyped()) {
            FoldStyle::Never
        } else {
            FoldStyle::Fit
        };
        ListStylist::new(self)
            .with_fold_style(fold_style)
            .process_list_impl(ctx, equation.to_untyped(), convert_math_padded)
            .print_doc(ListStyle {
                separator: "",
                delim: ("$", "$"),
                add_delim_space: is_block,
                tight_delim: !is_block,
                ..Default::default()
            })
    }

    pub(super) fn convert_math(&'a self, ctx: Context, math: Math<'a>) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(math.to_untyped()) {
            return res;
        }
        let ctx = ctx.suppress_breaks();
        if let Some(res) = self.try_convert_math_aligned(ctx, math) {
            return res;
        }
        self.convert_math_children(ctx, math.to_untyped().children())
    }

    pub(super) fn convert_math_children(
        &'a self,
        ctx: Context,
        math_children: impl Iterator<Item = &'a SyntaxNode>,
    ) -> ArenaDoc<'a> {
        let mut doc = self.arena.nil();
        let mut peek_hash = false;
        for node in math_children {
            let at_hash = peek_hash;
            peek_hash = false;
            if let Some(expr) = node.cast::<Expr>() {
                let ctx = ctx.with_mode_if(Mode::Code, at_hash);
                let expr_doc = self.convert_expr(ctx, expr);
                doc += expr_doc;
            } else if let Some(space) = node.cast::<Space>() {
                doc += self.convert_space(ctx, space);
            } else if node.kind() == SyntaxKind::Hash {
                doc += self.arena.text("#");
                peek_hash = true;
            } else if is_comment_node(node) {
                doc += self.convert_comment(ctx, node);
            } else {
                // may be LeftParen, RightParen
                doc += self.convert_trivia_untyped(node);
            }
        }
        doc
    }

    pub(super) fn convert_math_delimited(
        &'a self,
        ctx: Context,
        math_delimited: MathDelimited<'a>,
    ) -> ArenaDoc<'a> {
        let mut inner_nodes = math_delimited.to_untyped().children().as_slice();
        inner_nodes = &inner_nodes[1..inner_nodes.len() - 1];

        let mut has_open_linebreak = false;
        let mut has_close_space = false;
        let open_space = if let Some((first, rest)) = inner_nodes.split_first() {
            if first.kind() == SyntaxKind::Space {
                inner_nodes = rest;
                if first.text().has_linebreak() {
                    has_open_linebreak = true;
                    self.arena.hardline()
                } else {
                    self.arena.space()
                }
            } else {
                self.arena.nil()
            }
        } else {
            self.arena.nil()
        };
        let close_space = if let Some((last, rest)) = inner_nodes.split_last() {
            if last.kind() == SyntaxKind::Space {
                has_close_space = true;
                inner_nodes = rest;
                self.convert_space_untyped(ctx, last)
            } else {
                self.arena.nil()
            }
        } else {
            self.arena.nil()
        };
        let body = self.convert_flow_like_iter(ctx, inner_nodes.iter(), |ctx, node, _| {
            if let Some(math) = node.cast::<Math>() {
                let ctx = ctx.aligned(if has_open_linebreak && has_close_space {
                    AlignMode::Inner
                } else {
                    AlignMode::Never
                });
                FlowItem::tight(self.convert_math(ctx, math))
            } else if node.kind() == SyntaxKind::Space {
                // We can not arbitrarily break line here, as it may become ugly.
                FlowItem::tight(self.convert_space_untyped(ctx, node))
            } else {
                FlowItem::none()
            }
        });
        let open = self.convert_expr(ctx, math_delimited.open());
        let close = self.convert_expr(ctx, math_delimited.close());
        (self.indent(open_space + body) + close_space).enclose(open, close)
    }

    pub(super) fn convert_math_attach(
        &'a self,
        ctx: Context,
        math_attach: MathAttach<'a>,
    ) -> ArenaDoc<'a> {
        let mut peek_hashed_expr = false;
        self.convert_flow_like(ctx, math_attach.to_untyped(), |ctx, node, state| {
            let at_hashed_expr = peek_hashed_expr;
            if let Some(expr) = node.cast::<Expr>() {
                peek_hashed_expr = state.at_hash;
                FlowItem::tight(self.convert_expr(ctx, expr))
            } else if node.kind() == SyntaxKind::Underscore {
                peek_hashed_expr = false;
                FlowItem::tight(if at_hashed_expr {
                    self.convert_literal(" _")
                } else {
                    self.convert_literal("_")
                })
            } else if node.kind() == SyntaxKind::Hat {
                peek_hashed_expr = false;
                FlowItem::tight(self.convert_literal("^"))
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_math_primes(
        &'a self,
        _ctx: Context,
        math_primes: MathPrimes<'a>,
    ) -> ArenaDoc<'a> {
        self.arena.text("'".repeat(math_primes.count()))
    }

    pub(super) fn convert_math_frac(
        &'a self,
        ctx: Context,
        math_frac: MathFrac<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, math_frac.to_untyped(), |ctx, node, _| {
            if let Some(expr) = node.cast::<Expr>() {
                FlowItem::spaced(self.convert_expr(ctx, expr))
            } else if let Some(space) = node.cast::<Space>() {
                FlowItem::tight(self.convert_space(ctx, space))
            } else {
                FlowItem::tight(self.convert_trivia_untyped(node))
            }
        })
    }

    pub(super) fn convert_math_root(
        &'a self,
        ctx: Context,
        math_root: MathRoot<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, math_root.to_untyped(), |ctx, node, _| {
            if let Some(expr) = node.cast::<Expr>() {
                FlowItem::tight(self.convert_expr(ctx, expr))
            } else if node.kind() == SyntaxKind::Space {
                FlowItem::none()
            } else {
                FlowItem::tight(self.convert_trivia_untyped(node))
            }
        })
    }
}
