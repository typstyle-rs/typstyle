use itertools::Itertools;
use typst_syntax::{ast::*, SyntaxKind, SyntaxNode};

use super::{
    context::AlignMode,
    layout::{
        flow::FlowItem,
        list::{ListStyle, ListStylist},
        plain::PlainStylist,
    },
    prelude::*,
    style::FoldStyle,
    table, Context, Mode, PrettyPrinter,
};
use crate::{ext::StrExt, pretty::args};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_func_call(
        &'a self,
        ctx: Context,
        func_call: FuncCall<'a>,
    ) -> ArenaDoc<'a> {
        if func_call.callee().to_untyped().kind() == SyntaxKind::FieldAccess {
            if let Some(res) = self.try_convert_dot_chain(ctx, func_call.to_untyped()) {
                return res;
            }
        }
        self.convert_func_call_plain(ctx, func_call)
    }

    pub(super) fn convert_func_call_plain(
        &'a self,
        ctx: Context,
        func_call: FuncCall<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_expr(ctx, func_call.callee())
            + self.convert_func_call_args(ctx, func_call, func_call.args())
    }

    pub(super) fn convert_func_call_as_table(
        &'a self,
        ctx: Context,
        func_call: FuncCall<'a>,
        columns: usize,
    ) -> ArenaDoc<'a> {
        let args = func_call.args();
        let has_parenthesized_args = args::has_parenthesized_args(args);
        self.convert_expr(ctx, func_call.callee())
            + self.convert_table(ctx, func_call, columns)
            + self.convert_additional_args(ctx, args, has_parenthesized_args)
    }

    fn convert_func_call_args(
        &'a self,
        ctx: Context,
        func_call: FuncCall<'a>,
        args: Args<'a>,
    ) -> ArenaDoc<'a> {
        if ctx.mode.is_math() {
            return self.convert_args_in_math(ctx, args);
        };

        let mut doc = self.arena.nil();
        let has_parenthesized_args = args::has_parenthesized_args(args);
        if table::is_table(func_call) {
            if let Some(table) = self.try_convert_table(ctx, func_call) {
                doc += table;
            } else if has_parenthesized_args {
                doc += self.convert_parenthesized_args_as_list(ctx, args);
            }
        } else if has_parenthesized_args {
            doc += self.convert_parenthesized_args(ctx, args);
        };
        doc + self.convert_additional_args(ctx, args, has_parenthesized_args)
    }

    pub(super) fn convert_args(&'a self, ctx: Context, args: Args<'a>) -> ArenaDoc<'a> {
        let has_parenthesized_args = args::has_parenthesized_args(args);
        let parenthesized = if has_parenthesized_args {
            self.convert_parenthesized_args(ctx, args)
        } else {
            self.arena.nil()
        };
        parenthesized + self.convert_additional_args(ctx, args, has_parenthesized_args)
    }

    pub(super) fn convert_parenthesized_args(
        &'a self,
        ctx: Context,
        args: Args<'a>,
    ) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        let get_children = || {
            args.to_untyped()
                .children()
                .take_while(|it| it.kind() != SyntaxKind::RightParen)
        };
        let pargs = args::get_parenthesized_args(args).collect_vec();

        let fold_style = match self.get_fold_style(ctx, args) {
            _ if pargs.is_empty() => FoldStyle::Always,
            FoldStyle::Always => FoldStyle::Always,
            _ if ctx.break_suppressed && pargs.len() == 1 => FoldStyle::Always,
            _ if ctx.break_suppressed => suggest_fold_style_for_args(&pargs, FoldStyle::Fit),
            fold_style => suggest_fold_style_for_args(&pargs, fold_style),
        };

        ListStylist::new(self)
            .keep_linebreak(self.config.blank_lines_upper_bound)
            .with_fold_style(fold_style)
            .process_iterable_impl(ctx, get_children(), |ctx, child| {
                // We should ignore additional args here.
                child.cast().map(|arg| self.convert_arg(ctx, arg))
            })
            .print_doc(ListStyle {
                ..Default::default()
            })
    }

    fn convert_parenthesized_args_as_list(&'a self, ctx: Context, args: Args<'a>) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        let inner = PlainStylist::new(self)
            .process_iterable(
                ctx,
                args::get_parenthesized_args_untyped(args),
                |ctx, child| self.convert_arg(ctx, child),
            )
            .print_doc();
        inner.nest(self.config.tab_spaces as isize).parens()
    }

    fn convert_args_in_math(&'a self, ctx: Context, args: Args<'a>) -> ArenaDoc<'a> {
        // strip spaces
        let mut peek_linebreak = false;
        let children = {
            let children = args.to_untyped().children().as_slice();
            let i = children
                .iter()
                .position(|child| {
                    if child.kind() == SyntaxKind::Space {
                        peek_linebreak = child.text().has_linebreak();
                    }
                    !matches!(child.kind(), SyntaxKind::LeftParen | SyntaxKind::Space)
                })
                .expect("invariant: args should have right paren");
            let j = children
                .iter()
                .rposition(|child| {
                    !matches!(child.kind(), SyntaxKind::RightParen | SyntaxKind::Space)
                })
                .expect("invariant: args should have left paren");
            if i > j {
                children[0..0].iter()
            } else {
                children[i..=j].iter()
            }
        };

        let mut peek_hashed_arg = false;
        let inner = self.convert_flow_like_iter(ctx, children, |ctx, child, _| {
            let at_hashed_arg = peek_hashed_arg;
            let at_linebreak = peek_linebreak;
            peek_hashed_arg = false;
            peek_linebreak = false;
            match child.kind() {
                SyntaxKind::Comma => FlowItem::tight_spaced(self.arena.text(",")),
                SyntaxKind::Semicolon => {
                    // We should avoid the semicolon counted the terminator of the previous hashed arg.
                    FlowItem::new(self.arena.text(";"), at_hashed_arg, true)
                }
                SyntaxKind::Space => {
                    peek_hashed_arg = at_hashed_arg;
                    if child.text().has_linebreak() {
                        peek_linebreak = true;
                        FlowItem::tight(self.arena.hardline())
                    } else {
                        FlowItem::none()
                    }
                }
                _ => {
                    if let Some(arg) = child.cast::<Arg>() {
                        if is_ends_with_hashed_expr(arg.to_untyped().children()) {
                            peek_hashed_arg = true;
                        }
                        let ctx = ctx.aligned(
                            if at_linebreak || arg.to_untyped().kind() == SyntaxKind::MathDelimited
                            {
                                AlignMode::Inner
                            } else {
                                AlignMode::Never
                            },
                        );
                        FlowItem::spaced(self.convert_arg(ctx, arg))
                    } else {
                        FlowItem::none()
                    }
                }
            }
        });
        if self.attr_store.is_multiline(args.to_untyped()) {
            ((self.arena.line_() + inner).nest(self.config.tab_spaces as isize)
                + self.arena.line_())
            .group()
            .parens()
        } else {
            inner.parens()
        }
    }

    /// Handle additional content blocks
    fn convert_additional_args(
        &'a self,
        ctx: Context,
        args: Args<'a>,
        has_paren: bool,
    ) -> ArenaDoc<'a> {
        let args = args
            .to_untyped()
            .children()
            .skip_while(|node| {
                if has_paren {
                    node.kind() != SyntaxKind::RightParen
                } else {
                    node.kind() != SyntaxKind::ContentBlock
                }
            })
            .filter_map(|node| node.cast::<ContentBlock>());
        self.arena
            .concat(args.map(|arg| self.convert_content_block(ctx, arg)))
    }
}

fn is_ends_with_hashed_expr(mut children: std::slice::Iter<'_, SyntaxNode>) -> bool {
    children.next_back().is_some_and(|it| it.is::<Expr>())
        && children
            .next_back()
            .is_some_and(|it| it.kind() == SyntaxKind::Hash)
}

/// Suggests a folding style (`FoldStyle`) for parenthesized function-call arguments
/// based on their structure and content.
///
/// Rules:
/// 1. If the argument list is empty, default to `FoldStyle::Always`.
/// 2. If there is exactly one argument and it is a code block, always fold it.
/// 3. If the last argument is "combinable" (e.g., nested calls, arrays, dictionaries,
///    or parenthesized groups), all preceding arguments are simple (not blocks),
///    and there is no conflict with the last argument (i.e., both are arrays or dictionaries with items)
///    use compact layout (`FoldStyle::Compact`).
/// 4. Otherwise no folding is suggested (`None`).
///
/// # Parameters
/// - `pargs`: A slice of parenthesized arguments to analyze.
/// -
///
/// # Returns
/// - The new fold style
fn suggest_fold_style_for_args(pargs: &[Arg], hint: FoldStyle) -> FoldStyle {
    let Some((&last, initials)) = pargs.split_last() else {
        return FoldStyle::Always;
    };
    let last_expr = args::unwrap_expr_deep(last);

    if !args::is_combinable(last_expr) {
        return hint;
    }

    // If there’s only one argument and it’s a code block, always fold.
    if initials.is_empty() {
        // TODO: only return FoldStyle::Compact when we have better Union support,
        // so that only the length of the first line matters.
        return if matches!(last_expr, Expr::FuncCall(_)) {
            FoldStyle::Compact
        } else {
            FoldStyle::Always
        };
    }
    if hint == FoldStyle::Never {
        return hint;
    }

    let last_expr_kind = last_expr.to_untyped().kind();
    if initials.iter().any(|&arg| {
        // returns true if bad
        let expr = args::unwrap_expr_deep(arg);
        match expr {
            expr if args::is_blocky(expr) => true,
            _ if expr.to_untyped().kind() != last_expr_kind => false,
            Expr::Array(array) if array.items().next().is_some() => true,
            Expr::Dict(dict) if dict.items().next().is_some() => true,
            _ => false,
        }
    }) {
        return hint;
    }

    FoldStyle::Compact
}
