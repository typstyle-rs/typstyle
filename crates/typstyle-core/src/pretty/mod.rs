//! The core pretty-printing machinery: converts a Typst syntax tree into a
//! [`prettyless`] document.
//!
//! Conversions live in per-construct modules (`markup`, `math`, `code_*`, ...) and are
//! driven by the generic layout engines in the `layout` module. The [`context`] module
//! carries the per-traversal state, and [`style`] holds source-flavored style decisions.

pub mod context;
pub mod prelude;
pub mod style;

mod args;
mod code_chain;
mod code_flow;
mod code_list;
mod code_misc;
mod comment;
mod func_call;
mod import;
mod layout;
mod markup;
mod math;
mod math_align;
mod parened_expr;
mod table;
mod text;
mod util;

pub use context::{Context, Mode};
pub(crate) use markup::MarkupScope;
use prelude::*;
use prettyless::{IndentationPolicy, RenderOptions};
use style::{FoldStyle, is_multiline_flavored};
use typst_syntax::{SyntaxNode, ast::*};

use crate::{AttrStore, Config, Error};

/// The core pretty printer that converts a Typst syntax tree into a pretty document.
///
/// It holds the style [`Config`], the [`AttrStore`] for format-disabled regions, and the
/// document [`Arena`] that all produced documents are allocated in.
pub struct PrettyPrinter<'a> {
    config: Config,
    attr_store: AttrStore,
    arena: Arena<'a>,
}

impl<'a> PrettyPrinter<'a> {
    /// Creates a new printer with the given style configuration and node attributes.
    pub fn new(config: Config, attr_store: AttrStore) -> Self {
        Self {
            config,
            attr_store,
            arena: Arena::new(),
        }
    }

    /// Returns the style configuration of this printer.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Builds the render options for rendering documents produced by this printer.
    ///
    /// Lines are wrapped at [`Config::max_width`] columns, and indentation is written
    /// lazily (deferred) so that blank lines and trailing breaks carry no trailing spaces.
    pub(crate) fn render_options(&self) -> RenderOptions {
        RenderOptions::new(self.config.max_width)
            .with_indentation_policy(IndentationPolicy::Deferred)
    }

    /// Typed counterpart of [`Self::get_fold_style_untyped`].
    fn get_fold_style(&self, ctx: Context, node: impl AstNode<'a>) -> FoldStyle {
        self.get_fold_style_untyped(ctx, node.to_untyped())
    }

    /// Determines how the items of a list-like node should be laid out.
    ///
    /// A node with a line break before its first item ([`is_multiline_flavored`]) keeps
    /// one item per line ([`FoldStyle::Never`]); otherwise, items are folded if they fit
    /// on one line ([`FoldStyle::Fit`]). When line breaks are suppressed in the
    /// [`Context`], folding is preferred: multiline nodes fall back to
    /// [`FoldStyle::Fit`], and others are always folded ([`FoldStyle::Always`]).
    fn get_fold_style_untyped(&self, ctx: Context, node: &'a SyntaxNode) -> FoldStyle {
        let is_multiline = is_multiline_flavored(node);
        if ctx.break_suppressed {
            return if is_multiline {
                FoldStyle::Fit
            } else {
                FoldStyle::Always
            };
        }
        if is_multiline {
            FoldStyle::Never
        } else {
            FoldStyle::Fit
        }
    }
}

/// Utilities
impl<'a> PrettyPrinter<'a> {
    /// Nests the document by one indentation level ([`Config::tab_spaces`] spaces).
    pub(crate) fn indent(&'a self, doc: ArenaDoc<'a>) -> ArenaDoc<'a> {
        doc.nest(self.config.tab_spaces as isize)
    }

    /// Wraps a document into an indented block delimited by line breaks.
    ///
    /// The leading break and the document are nested together, so the block body sits
    /// one indentation level deeper than the surrounding text.
    pub(crate) fn block_indent(&'a self, doc: ArenaDoc<'a>) -> ArenaDoc<'a> {
        self.indent(self.arena.line_() + doc) + self.arena.line_()
    }
}

impl<'a> PrettyPrinter<'a> {
    /// Returns a verbatim conversion of the node if formatting is disabled for it
    /// (e.g., by a `@typstyle off` comment), or [`None`] otherwise.
    fn check_disabled(&'a self, node: &'a SyntaxNode) -> Option<ArenaDoc<'a>> {
        if self.attr_store.is_format_disabled(node) {
            Some(self.convert_verbatim_untyped(node))
        } else {
            None
        }
    }

    /// Converts a node by reproducing its full source text as is.
    ///
    /// For inner or lead nodes only. See [`Self::convert_verbatim_untyped`].
    fn convert_verbatim(&'a self, node: impl AstNode<'a>) -> ArenaDoc<'a> {
        self.convert_verbatim_untyped(node.to_untyped())
    }

    /// Converts a node by reproducing its full source text, including all of its
    /// descendants, as is.
    ///
    /// For inner or lead nodes only.
    fn convert_verbatim_untyped(&'a self, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        let text = node.full_text();
        self.arena.multiline_text(text.to_string())
    }

    /// Produce a document that reproduces the given text as is, including all line
    /// breaks.
    pub(crate) fn convert_verbatim_text(&'a self, text: &'a str) -> ArenaDoc<'a> {
        self.arena.multiline_text(text)
    }

    /// Converts a leaf node by emitting its text as is.
    ///
    /// For leaf nodes only. See [`Self::convert_trivia_untyped`].
    fn convert_trivia(&'a self, node: impl AstNode<'a>) -> ArenaDoc<'a> {
        self.convert_trivia_untyped(node.to_untyped())
    }

    /// Emits the leaf text of a node as is.
    ///
    /// For leaf nodes only.
    fn convert_trivia_untyped(&'a self, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        self.arena.text(node.leaf_text().as_str())
    }

    /// Converts a root syntax node into a document, starting from the given [`Mode`].
    ///
    /// The node is dispatched by its kind: it must be a [`Markup`], [`Code`], [`Expr`],
    /// or [`Pattern`]. Returns [`Error::SyntaxError`] if it is none of these.
    pub fn try_convert_with_mode(
        &'a self,
        node: &'a SyntaxNode,
        mode: Mode,
    ) -> Result<ArenaDoc<'a>, Error> {
        self.try_convert_with_scope(node, mode, MarkupScope::Document)
    }

    /// Like [`Self::try_convert_with_mode`], but with an explicit [`MarkupScope`] for
    /// the top-level markup.
    ///
    /// Partial formatting uses [`MarkupScope::Fragment`] to format a selection within
    /// markup while retaining its own whitespace boundaries.
    pub(crate) fn try_convert_with_scope(
        &'a self,
        node: &'a SyntaxNode,
        mode: Mode,
        markup_scope: MarkupScope,
    ) -> Result<ArenaDoc<'a>, Error> {
        let ctx = Context::default().with_mode(mode);
        let doc = if let Some(markup) = node.cast() {
            self.convert_markup_with_scope(ctx, markup, markup_scope)
        } else if let Some(code) = node.cast() {
            self.convert_code(ctx, code)
        } else if let Some(expr) = node.cast() {
            self.convert_expr(ctx, expr)
        } else if let Some(pattern) = node.cast() {
            self.convert_pattern(ctx, pattern)
        } else {
            return Err(Error::SyntaxError);
        };
        Ok(doc)
    }

    /// Converts an expression, checking first whether its formatting is disabled.
    ///
    /// Disabled expressions (e.g., marked by `@typstyle off`) are converted verbatim;
    /// otherwise they are dispatched by kind.
    pub fn convert_expr(&'a self, ctx: Context, expr: Expr<'a>) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(expr.to_untyped()) {
            return res;
        }
        self.convert_expr_impl(ctx, expr)
    }

    /// Dispatches an expression to its kind-specific conversion.
    ///
    /// Unlike [`Self::convert_expr`], this does not check for format-disabled regions.
    fn convert_expr_impl(&'a self, ctx: Context, expr: Expr<'a>) -> ArenaDoc<'a> {
        match expr {
            Expr::Text(t) => self.convert_text(t),
            Expr::Space(s) => self.convert_space(ctx, s),
            Expr::Linebreak(b) => self.convert_trivia(b),
            Expr::Parbreak(b) => self.convert_parbreak(b),
            Expr::Escape(e) => self.convert_trivia(e),
            Expr::Shorthand(s) => self.convert_trivia(s),
            Expr::SmartQuote(s) => self.convert_trivia(s),
            Expr::Strong(s) => self.convert_strong(ctx, s),
            Expr::Emph(e) => self.convert_emph(ctx, e),
            Expr::Raw(r) => self.convert_raw(ctx, r),
            Expr::Link(l) => self.convert_trivia(l),
            Expr::Label(l) => self.convert_trivia(l),
            Expr::Ref(r) => self.convert_ref(ctx, r),
            Expr::Heading(h) => self.convert_heading(ctx, h),
            Expr::ListItem(l) => self.convert_list_item(ctx, l),
            Expr::EnumItem(e) => self.convert_enum_item(ctx, e),
            Expr::TermItem(t) => self.convert_term_item(ctx, t),
            Expr::Equation(e) => self.convert_equation(ctx, e),
            Expr::Math(m) => self.convert_math(ctx, m),
            Expr::MathText(math_text) => self.convert_trivia(math_text),
            Expr::MathIdent(mi) => self.convert_trivia(mi),
            Expr::MathFieldAccess(mfa) => self.convert_verbatim(mfa),
            Expr::MathAlignPoint(map) => self.convert_trivia(map),
            Expr::MathCall(mc) => self.convert_math_call(ctx, mc),
            Expr::MathDelimited(md) => self.convert_math_delimited(ctx, md),
            Expr::MathAttach(ma) => self.convert_math_attach(ctx, ma),
            Expr::MathPrimes(mp) => self.convert_math_primes(ctx, mp),
            Expr::MathFrac(mf) => self.convert_math_frac(ctx, mf),
            Expr::MathRoot(mr) => self.convert_math_root(ctx, mr),
            Expr::MathShorthand(ms) => self.convert_trivia(ms),
            Expr::Ident(i) => self.convert_ident(i),
            Expr::None(_) => self.convert_literal("none"),
            Expr::Auto(_) => self.convert_literal("auto"),
            Expr::Bool(b) => self.convert_trivia(b),
            Expr::Int(i) => self.convert_trivia(i),
            Expr::Float(f) => self.convert_trivia(f),
            Expr::Numeric(n) => self.convert_trivia(n),
            Expr::Str(s) => self.convert_trivia(s),
            Expr::CodeBlock(c) => self.convert_code_block(ctx, c),
            Expr::ContentBlock(c) => self.convert_content_block(ctx, c),
            Expr::Parenthesized(p) => self.convert_parenthesized(ctx, p),
            Expr::Array(a) => self.convert_array(ctx, a),
            Expr::Dict(d) => self.convert_dict(ctx, d),
            Expr::Unary(u) => self.convert_unary(ctx, u),
            Expr::Binary(b) => self.convert_binary(ctx, b),
            Expr::FieldAccess(fa) => self.convert_field_access(ctx, fa),
            Expr::FuncCall(fc) => self.convert_func_call(ctx, fc),
            Expr::Closure(c) => self.convert_closure(ctx, c),
            Expr::LetBinding(l) => self.convert_let_binding(ctx, l),
            Expr::DestructAssignment(da) => self.convert_destruct_assignment(ctx, da),
            Expr::SetRule(s) => self.convert_set_rule(ctx, s),
            Expr::ShowRule(s) => self.convert_show_rule(ctx, s),
            Expr::Contextual(c) => self.convert_contextual(ctx, c),
            Expr::Conditional(c) => self.convert_conditional(ctx, c),
            Expr::WhileLoop(w) => self.convert_while_loop(ctx, w),
            Expr::ForLoop(f) => self.convert_for_loop(ctx, f),
            Expr::ModuleImport(i) => self.convert_import(ctx, i),
            Expr::ModuleInclude(i) => self.convert_include(ctx, i),
            Expr::LoopBreak(_) => self.convert_literal("break"),
            Expr::LoopContinue(_) => self.convert_literal("continue"),
            Expr::FuncReturn(r) => self.convert_return(ctx, r),
        }
    }

    /// Emits a fixed keyword, such as `none` or `break`, as plain text.
    fn convert_literal(&'a self, literal: &'a str) -> ArenaDoc<'a> {
        self.arena.text(literal)
    }
}
