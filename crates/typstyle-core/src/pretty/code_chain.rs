//! Conversion of field-access and binary-operator chains.

use itertools::Itertools;
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{
    Context,
    layout::chain::{ChainStyle, ChainStylist, iterate_deep_nodes},
    prelude::*,
    util::has_comment_children,
};
use crate::PrettyPrinter;

impl<'a> PrettyPrinter<'a> {
    /// Converts a field access, as a dot chain if possible.
    pub(super) fn convert_field_access(
        &'a self,
        ctx: Context,
        field_access: FieldAccess<'a>,
    ) -> ArenaDoc<'a> {
        if let Some(res) = self.try_convert_dot_chain(ctx, field_access.to_untyped()) {
            return res;
        }
        self.convert_field_access_plain(ctx, field_access)
    }

    /// Converts a field access as `target.field`, without chain handling.
    fn convert_field_access_plain(
        &'a self,
        ctx: Context,
        field_access: FieldAccess<'a>,
    ) -> ArenaDoc<'a> {
        // Comments within field access are not allowed outside code mode
        self.convert_expr(ctx, field_access.target())
            + self.arena.text(".")
            + self.convert_ident(field_access.field())
    }

    /// Convert the node as dot chain, if in code, or in markup with at least two FieldAccess and one FuncCall.
    pub(super) fn try_convert_dot_chain(
        &'a self,
        ctx: Context,
        node: &'a SyntaxNode,
    ) -> Option<ArenaDoc<'a>> {
        if ctx.break_suppressed {
            return None;
        }
        let mut dot_num = 0;
        let mut call_num = 0;
        let mut has_comment = false;
        let chain: Vec<&SyntaxNode> = resolve_dot_chain(node).collect_vec();
        for node in &chain {
            if node.kind() == SyntaxKind::FieldAccess {
                dot_num += 1;
            } else if node.kind() == SyntaxKind::FuncCall {
                call_num += 1;
            }
            if has_comment_children(node) {
                has_comment = true;
            }
        }
        if dot_num > 1
            && call_num == 1
            && !has_comment
            && let Some(res) = self.try_convert_dot_chain_plain(ctx, chain)
        {
            return Some(res);
        }
        if ctx.mode.is_markup() && dot_num > 1 && call_num > 0 {
            return Some(
                self.parenthesize_if_necessary(ctx, |ctx| self.convert_dot_chain(ctx, node)),
            );
        } else if ctx.mode.is_code() {
            return Some(self.convert_dot_chain(ctx, node));
        }
        None
    }

    /// Converts the special case `ident.ident(args)`, keeping the idents on one line.
    ///
    /// Returns [`None`] if the chain is not a [`FuncCall`] rooted at an [`Ident`], or if
    /// the estimated chain width reaches [`crate::Config::chain_width`].
    fn try_convert_dot_chain_plain(
        &'a self,
        ctx: Context,
        mut chain: Vec<&'a SyntaxNode>,
    ) -> Option<ArenaDoc<'a>> {
        chain.reverse();
        let func_call = chain.last()?.cast::<FuncCall>()?;
        let ident = chain[0].cast::<Ident>()?;

        // If the chain idents are long enough, do not put them in one line.
        let estimated_len = ident.get().len()
            + chain
                .iter()
                .skip(1)
                .map(|child| {
                    child
                        .cast::<FieldAccess>()
                        .map(|field_access| field_access.field().get().len() + 1)
                        .unwrap_or_default()
                })
                .sum::<usize>();
        if estimated_len >= self.config.chain_width() {
            return None;
        }

        let mut doc = self.convert_ident(ident);
        for child in chain {
            if let Some(field_access) = child.cast::<FieldAccess>() {
                doc += self.arena.text(".") + self.convert_ident(field_access.field());
            }
        }
        doc += self.convert_args_of_func(ctx, func_call);
        Some(doc)
    }

    /// Converts the node as a dot chain using [`ChainStylist`].
    ///
    /// A chain with a single field access and no comments is kept on one line
    /// ([`ChainStyle::no_break_single`]).
    fn convert_dot_chain(&'a self, ctx: Context, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        ChainStylist::new(self)
            .process_resolved(
                ctx,
                resolve_dot_chain(node),
                |node| node.kind() == SyntaxKind::FieldAccess,
                |child| {
                    if child.kind() == SyntaxKind::Dot {
                        Some(self.arena.text("."))
                    } else {
                        None
                    }
                },
                |_ctx, child| child.cast().map(|ident| self.convert_ident(ident)),
                |ctx, node| {
                    if let Some(func_call) = node.cast::<FuncCall>() {
                        // There is no comment allowed, so we can directly convert args.
                        Some(self.convert_args_of_func(ctx, func_call))
                    } else {
                        node.cast().map(|expr| self.convert_expr(ctx, expr))
                    }
                },
            )
            .print_doc(ChainStyle {
                no_break_single: true,
                ..Default::default()
            })
    }

    /// Converts a binary expression as a chain of its same-precedence operands.
    ///
    /// Operators are surrounded by spaces; the chain breaks across lines unless it fits.
    pub(super) fn convert_binary_chain(&'a self, ctx: Context, binary: Binary<'a>) -> ArenaDoc<'a> {
        let op = binary.op();
        let prec = op.precedence();
        ChainStylist::new(self)
            .process_resolved(
                ctx,
                resolve_binary_chain(binary),
                |node| {
                    node.cast::<Binary>()
                        .is_some_and(|binary| binary.op().precedence() == prec)
                },
                |child| {
                    if child.kind() == SyntaxKind::In && op == BinOp::NotIn {
                        Some(self.arena.text(op.as_str()))
                    } else {
                        BinOp::from_kind(child.kind()).map(|op| self.arena.text(op.as_str()))
                    }
                },
                |ctx, child| child.cast().map(|expr| self.convert_expr(ctx, expr)),
                |ctx, node| node.cast().map(|expr| self.convert_expr(ctx, expr)),
            )
            .print_doc(ChainStyle {
                space_around_op: true,
                ..Default::default()
            })
    }
}

/// Resolves a field-access and function-call chain into its nodes, outermost first.
///
/// Follows the target of [`FieldAccess`] and the callee of [`FuncCall`] down to the
/// innermost expression.
pub(super) fn resolve_dot_chain(node: &SyntaxNode) -> impl Iterator<Item = &SyntaxNode> {
    iterate_deep_nodes(node, |current| {
        if let Some(field_access) = current.cast::<FieldAccess>() {
            Some(field_access.target().to_untyped())
        } else if let Some(func_call) = current.cast::<FuncCall>() {
            Some(func_call.callee().to_untyped())
        } else {
            None
        }
    })
}

/// Resolves a binary expression into its nodes, outermost first.
///
/// Only [`Binary`] nodes with the same operator precedence as the root are descended
/// into, so differently binding subexpressions stay intact.
pub(super) fn resolve_binary_chain(binary: Binary<'_>) -> impl Iterator<Item = &'_ SyntaxNode> {
    let prec = binary.op().precedence();
    iterate_deep_nodes(binary.to_untyped(), move |current| {
        if let Some(binary) = current.cast::<Binary>()
            && binary.op().precedence() == prec
        {
            return Some(binary.lhs().to_untyped());
        }
        None
    })
}
