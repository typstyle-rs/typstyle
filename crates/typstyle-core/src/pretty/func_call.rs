use itertools::Itertools;
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{
    Context, Mode, PrettyPrinter,
    context::AlignMode,
    layout::{
        flow::FlowItem,
        list::{ListStyle, ListStylist},
        plain::PlainStylist,
    },
    prelude::*,
    style::FoldStyle,
    table,
};
use crate::{ext::StrExt, pretty::args};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_func_call(
        &'a self,
        ctx: Context,
        func_call: FuncCall<'a>,
    ) -> ArenaDoc<'a> {
        if func_call.callee().to_untyped().kind() == SyntaxKind::FieldAccess
            && let Some(res) = self.try_convert_dot_chain(ctx, func_call.to_untyped())
        {
            return res;
        }
        self.convert_expr(ctx, func_call.callee())
            + if ctx.mode.is_math() {
                self.convert_args_in_math(ctx, func_call.args())
            } else {
                self.convert_args_of_func(ctx, func_call)
            }
    }

    pub(super) fn convert_args_of_func(
        &'a self,
        ctx: Context,
        func_call: FuncCall<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_args(ctx, func_call.args(), |nodes| {
            self.convert_parenthesized_args_of_func(ctx, func_call, nodes)
        })
    }

    /// The most thorough converter for parenthesized args, considering tables.
    fn convert_parenthesized_args_of_func(
        &'a self,
        ctx: Context,
        func_call: FuncCall<'a>,
        paren_nodes: &'a [SyntaxNode],
    ) -> ArenaDoc<'a> {
        if table::is_table(func_call) {
            if let Some(table) = self.try_convert_table(ctx, func_call, paren_nodes) {
                table
            } else {
                self.convert_parenthesized_args_as_list(ctx, paren_nodes)
            }
        } else {
            self.convert_parenthesized_args_normal(ctx, func_call.args(), paren_nodes)
        }
    }

    pub(super) fn convert_args(
        &'a self,
        ctx: Context,
        args: Args<'a>,
        parenthesized_nodes_handler: impl FnOnce(&'a [SyntaxNode]) -> ArenaDoc<'a>,
    ) -> ArenaDoc<'a> {
        let (paren_nodes, trailing_nodes) = args::split_paren_args(args);

        // 1) parenthesized args (if any)
        let paren_doc = match paren_nodes {
            [] | [_] => self.arena.nil(),    // no parentheses at all
            [_, _] => self.arena.text("()"), // exactly `(` `)` → emit "()"
            [_, inner @ .., _] => parenthesized_nodes_handler(inner), // at least one element between the parens
        };

        // 2) trailing content args (whatever comes after `)`)
        let trailing_doc = self.convert_trailing_content_args(ctx, trailing_nodes);

        paren_doc + trailing_doc
    }

    /// Handle additional content blocks
    fn convert_trailing_content_args(
        &'a self,
        ctx: Context,
        nodes: &'a [SyntaxNode],
    ) -> ArenaDoc<'a> {
        self.arena.concat(
            nodes
                .iter()
                .filter_map(SyntaxNode::cast)
                .map(|arg| self.convert_content_block(ctx, arg)),
        )
    }

    pub(super) fn convert_parenthesized_args_normal(
        &'a self,
        ctx: Context,
        args: Args<'a>,
        paren_nodes: &'a [SyntaxNode],
    ) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        let pargs = paren_nodes
            .iter()
            .filter_map(SyntaxNode::cast)
            .collect_vec();

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
            .process_iterable_impl(ctx, paren_nodes.iter(), |ctx, child| {
                // We should ignore additional args here.
                child.cast().map(|arg| self.convert_arg(ctx, arg))
            })
            .print_doc(ListStyle {
                ..Default::default()
            })
    }

    fn convert_parenthesized_args_as_list(
        &'a self,
        ctx: Context,
        paren_nodes: &'a [SyntaxNode],
    ) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        let inner = PlainStylist::new(self)
            .process_iterable(ctx, paren_nodes.iter(), |ctx, child| {
                self.convert_arg(ctx, child)
            })
            .print_doc();
        self.indent(inner).parens()
    }

    /// Args in math do not have trailing content args.
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
            self.block_indent(inner).group().parens()
        } else {
            inner.parens()
        }
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
/// Rules (ordered):
/// 1. **Empty argument list:**
///    - -> `FoldStyle::Always`
/// 2. **Single combinable argument:**
///    - The only argument is a function call -> `FoldStyle::Compact`
///    - Otherwise -> `FoldStyle::Always`
/// 3. **Multiple arguments without multiline flavor, combinable last argument:**
///    - The last argument is "combinable" (e.g., nested call, array, dictionary, or parenthesized group),
///      and all preceding arguments are simple (not blocks),
///      and the last argument must not be an array (or dictionary) if any previous argument is also an array (or dictionary)
///      -> `FoldStyle::Compact`
/// 4. **Otherwise:**
///    - -> the original fold style
///
/// # Parameters
/// - `pargs`: A slice of parenthesized arguments to analyze.
/// - `hint`: The fallback fold style to use if no rule matches.
///
/// # Returns
/// - The suggested `FoldStyle`
fn suggest_fold_style_for_args(pargs: &[Arg], hint: FoldStyle) -> FoldStyle {
    let Some((&last, initials)) = pargs.split_last() else {
        return FoldStyle::Always; // This should have been covered by call site.
    };
    let last_expr = args::unwrap_expr_deep(last);

    if !args::is_combinable(last_expr) {
        return hint;
    }

    // If there’s only one argument and it’s a code block, always fold.
    // Otherwise, respect its flavor.
    if initials.is_empty() {
        return if matches!(last_expr, Expr::CodeBlock(_)) {
            FoldStyle::Always
        } else if hint == FoldStyle::Never {
            FoldStyle::Never
        } else {
            FoldStyle::Compact
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
