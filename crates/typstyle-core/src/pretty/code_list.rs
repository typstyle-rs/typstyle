use typst_syntax::{SyntaxKind, ast::*};

use super::{
    Context, Mode, PrettyPrinter,
    layout::list::{ListStyle, ListStylist},
    prelude::*,
    style::FoldStyle,
    util::{has_comment_children, is_only_one_and},
};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_code_block(
        &'a self,
        ctx: Context,
        code_block: CodeBlock<'a>,
    ) -> ArenaDoc<'a> {
        if self
            .attr_store
            .is_format_disabled(code_block.body().to_untyped())
        {
            return self.convert_verbatim(code_block);
        }

        let ctx = ctx.with_mode(Mode::Code);

        let mut nodes = vec![];
        for child in code_block.to_untyped().children() {
            if let Some(code) = child.cast::<Code>() {
                nodes.extend(code.to_untyped().children());
            } else {
                nodes.push(child);
            }
        }

        let can_fold = code_block.body().exprs().count() <= 1
            && !has_comment_children(code_block.to_untyped());
        ListStylist::new(self)
            .disallow_front_comment()
            .with_fold_style(if can_fold {
                self.get_fold_style(ctx, code_block)
            } else {
                FoldStyle::Never
            })
            .keep_linebreak(self.config.blank_lines_upper_bound)
            .process_iterable(ctx, nodes.into_iter(), |ctx, expr| {
                self.convert_expr(ctx, expr)
            })
            .print_doc(ListStyle {
                separator: "",
                delim: ("{", "}"),
                add_delim_space: true,
                ..Default::default()
            })
    }

    /// Only used for partial format.
    pub(super) fn convert_code(&'a self, ctx: Context, code: Code<'a>) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::Code);

        ListStylist::new(self)
            .disallow_front_comment()
            .with_fold_style(FoldStyle::Never)
            .keep_linebreak(self.config.blank_lines_upper_bound)
            .process_iterable(ctx, code.to_untyped().children(), |ctx, expr| {
                self.convert_expr(ctx, expr)
            })
            .print_doc(ListStyle {
                separator: "",
                delim: ("", ""),
                tight_delim: true,
                no_indent: true,
                ..Default::default()
            })
    }

    pub(super) fn convert_parenthesized_impl(
        &'a self,
        ctx: Context,
        parenthesized: Parenthesized<'a>,
    ) -> ArenaDoc<'a> {
        // NOTE: This is a safe cast. The parentheses for patterns are all optional.
        // For safety, we don't remove parentheses around idents. See `paren-in-key.typ`.
        let expr = parenthesized.expr();
        let can_omit = (expr.is_literal()
            || matches!(
                expr.to_untyped().kind(),
                SyntaxKind::Array
                    | SyntaxKind::Dict
                    | SyntaxKind::Destructuring
                    | SyntaxKind::CodeBlock
                    | SyntaxKind::ContentBlock
            ))
            && !has_comment_children(parenthesized.to_untyped());

        ListStylist::new(self)
            .with_fold_style(self.get_fold_style(ctx, parenthesized))
            .process_list(ctx, parenthesized.to_untyped(), |ctx, node| {
                self.convert_pattern(ctx, node)
            })
            .print_doc(ListStyle {
                separator: "",
                omit_delim_flat: can_omit,
                ..Default::default()
            })
    }

    /// In math mode, we have `$fun(1, 2; 3, 4)$ == $fun(#(1, 2), #(3, 4))$`.
    pub(super) fn convert_array(&'a self, ctx: Context, array: Array<'a>) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        // Whether the array has parens.
        // This is also used to determine whether we need to add a trailing comma.
        // Note that we should not strip trailing commas in math.
        let is_explicit = array
            .to_untyped()
            .children()
            .next()
            .is_some_and(|child| child.kind() == SyntaxKind::LeftParen);
        let ends_with_comma = !is_explicit
            && array
                .to_untyped()
                .children()
                .last()
                .is_some_and(|child| child.kind() == SyntaxKind::Comma);

        ListStylist::new(self)
            .with_fold_style(self.get_fold_style(ctx, array))
            .keep_linebreak(self.config.blank_lines_upper_bound)
            .process_list(ctx, array.to_untyped(), |ctx, node| {
                self.convert_array_item(ctx, node)
            })
            .print_doc(ListStyle {
                add_trailing_sep_single: is_explicit,
                add_trailing_sep_always: ends_with_comma,
                delim: if is_explicit { ("(", ")") } else { ("", "") },
                tight_delim: !is_explicit,
                no_indent: !is_explicit,
                ..Default::default()
            })
    }

    pub(super) fn convert_dict(&'a self, ctx: Context, dict: Dict<'a>) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        let all_spread = dict.items().all(|item| matches!(item, DictItem::Spread(_)));

        ListStylist::new(self)
            .with_fold_style(self.get_fold_style(ctx, dict))
            .keep_linebreak(self.config.blank_lines_upper_bound)
            .process_list(ctx, dict.to_untyped(), |ctx, node| {
                self.convert_dict_item(ctx, node)
            })
            .print_doc(ListStyle {
                delim: (if all_spread { "(:" } else { "(" }, ")"),
                ..Default::default()
            })
    }

    pub(super) fn convert_destructuring(
        &'a self,
        ctx: Context,
        destructuring: Destructuring<'a>,
    ) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        let only_one_pattern = is_only_one_and(destructuring.items(), |it| {
            matches!(*it, DestructuringItem::Pattern(_))
        });

        ListStylist::new(self)
            .with_fold_style(self.get_fold_style(ctx, destructuring))
            .keep_linebreak(self.config.blank_lines_upper_bound)
            .process_list(ctx, destructuring.to_untyped(), |ctx, node| {
                self.convert_destructuring_item(ctx, node)
            })
            .always_fold_if(|| only_one_pattern)
            .print_doc(ListStyle {
                add_trailing_sep_single: only_one_pattern,
                ..Default::default()
            })
    }

    pub(super) fn convert_params(
        &'a self,
        ctx: Context,
        params: Params<'a>,
        is_unnamed: bool,
    ) -> ArenaDoc<'a> {
        // SAFETY: The param must be simple if the parens is optional.
        let ctx = ctx.with_mode(Mode::CodeCont);

        let is_single_simple = is_unnamed
            && is_only_one_and(params.children(), |it| {
                matches!(
                    *it,
                    Param::Pos(Pattern::Normal(_)) | Param::Pos(Pattern::Placeholder(_))
                )
            });

        ListStylist::new(self)
            .with_fold_style(self.get_fold_style(ctx, params))
            .keep_linebreak(self.config.blank_lines_upper_bound)
            .process_list(ctx, params.to_untyped(), |ctx, node| {
                self.convert_param(ctx, node)
            })
            .always_fold_if(|| is_single_simple)
            .print_doc(ListStyle {
                omit_delim_single: is_single_simple,
                ..Default::default()
            })
    }
}
