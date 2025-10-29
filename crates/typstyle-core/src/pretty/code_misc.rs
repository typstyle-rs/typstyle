use typst_syntax::ast::*;

use super::{Context, PrettyPrinter, prelude::*};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_ident(&'a self, ident: Ident<'a>) -> ArenaDoc<'a> {
        self.convert_trivia(ident)
    }

    pub(super) fn convert_array_item(
        &'a self,
        ctx: Context,
        array_item: ArrayItem<'a>,
    ) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(array_item.to_untyped()) {
            return res;
        }
        match array_item {
            ArrayItem::Pos(p) => self.convert_expr(ctx, p),
            ArrayItem::Spread(s) => self.convert_spread(ctx, s),
        }
    }

    pub(super) fn convert_dict_item(
        &'a self,
        ctx: Context,
        dict_item: DictItem<'a>,
    ) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(dict_item.to_untyped()) {
            return res;
        }
        match dict_item {
            DictItem::Named(n) => self.convert_named(ctx, n),
            DictItem::Keyed(k) => self.convert_keyed(ctx, k),
            DictItem::Spread(s) => self.convert_spread(ctx, s),
        }
    }

    pub(super) fn convert_arg(&'a self, ctx: Context, arg: Arg<'a>) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(arg.to_untyped()) {
            return res;
        }
        match arg {
            Arg::Pos(p) => self.convert_expr(ctx, p),
            Arg::Named(n) => self.convert_named(ctx, n),
            Arg::Spread(s) => self.convert_spread(ctx, s),
        }
    }

    pub(super) fn convert_param(&'a self, ctx: Context, param: Param<'a>) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(param.to_untyped()) {
            return res;
        }
        match param {
            Param::Pos(p) => self.convert_pattern(ctx, p),
            Param::Named(n) => self.convert_named(ctx, n),
            Param::Spread(s) => self.convert_spread(ctx, s),
        }
    }

    pub fn convert_pattern(&'a self, ctx: Context, pattern: Pattern<'a>) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(pattern.to_untyped()) {
            return res;
        }
        match pattern {
            Pattern::Normal(n) => self.convert_expr(ctx, n),
            Pattern::Placeholder(_) => self.convert_literal("_"),
            Pattern::Destructuring(d) => self.convert_destructuring(ctx, d),
            Pattern::Parenthesized(p) => self.convert_parenthesized(ctx, p),
        }
    }

    pub(super) fn convert_destructuring_item(
        &'a self,
        ctx: Context,
        destructuring_item: DestructuringItem<'a>,
    ) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(destructuring_item.to_untyped()) {
            return res;
        }
        match destructuring_item {
            DestructuringItem::Spread(s) => self.convert_spread(ctx, s),
            DestructuringItem::Named(n) => self.convert_named(ctx, n),
            DestructuringItem::Pattern(p) => self.convert_pattern(ctx, p),
        }
    }
}
