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
use prelude::*;
use style::{FoldStyle, is_multiline_flavored};
#[cfg(feature = "mapping")]
use typst_syntax::Source;
use typst_syntax::{SyntaxNode, ast::*};

#[cfg(feature = "mapping")]
use crate::ir_mapping::{IrAtomRecorder, IrAtomSource};
use crate::{AttrStore, Config, Error, ext::StrExt};

pub struct PrettyPrinter<'a> {
    config: Config,
    attr_store: AttrStore,
    arena: Arena<'a>,
    #[cfg(feature = "mapping")]
    source: Option<Source>,
    #[cfg(feature = "mapping")]
    ir_atom_recorder: std::cell::RefCell<IrAtomRecorder>,
}

impl<'a> PrettyPrinter<'a> {
    pub fn new(config: Config, attr_store: AttrStore) -> Self {
        Self {
            config,
            attr_store,
            arena: Arena::new(),
            #[cfg(feature = "mapping")]
            source: None,
            #[cfg(feature = "mapping")]
            ir_atom_recorder: std::cell::RefCell::new(IrAtomRecorder::default()),
        }
    }

    #[cfg(feature = "mapping")]
    pub fn new_with_source(config: Config, attr_store: AttrStore, source: Source) -> Self {
        Self {
            config,
            attr_store,
            arena: Arena::new(),
            source: Some(source),
            ir_atom_recorder: std::cell::RefCell::new(IrAtomRecorder::default()),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    fn get_fold_style(&self, ctx: Context, node: impl AstNode<'a>) -> FoldStyle {
        self.get_fold_style_untyped(ctx, node.to_untyped())
    }

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
    pub(crate) fn indent(&'a self, doc: ArenaDoc<'a>) -> ArenaDoc<'a> {
        doc.nest(self.config.tab_spaces as isize)
    }

    pub(crate) fn block_indent(&'a self, doc: ArenaDoc<'a>) -> ArenaDoc<'a> {
        self.indent(self.arena.line_() + doc) + self.arena.line_()
    }
}

impl<'a> PrettyPrinter<'a> {
    #[cfg(feature = "mapping")]
    pub(crate) fn reset_ir_mapping(&self) {
        self.ir_atom_recorder.borrow_mut().reset();
    }

    #[cfg(feature = "mapping")]
    pub(crate) fn snapshot_ir_atom_sources(&self) -> Vec<IrAtomSource> {
        self.ir_atom_recorder.borrow().snapshot_sources()
    }

    pub(crate) fn emit_source_text_untyped(
        &'a self,
        _node: &'a SyntaxNode,
        text: &'a str,
    ) -> ArenaDoc<'a> {
        #[cfg(feature = "mapping")]
        {
            let range = self
                .source
                .as_ref()
                .and_then(|source| source.range(_node.span()))
                .or_else(|| _node.span().range());
            if let Some(range) = range {
                let atom_id = self
                    .ir_atom_recorder
                    .borrow_mut()
                    .alloc_atom(range.start, range.end);
                return self.arena.text(text).tag(atom_id);
            }
        }

        self.arena.text(text)
    }

    fn check_disabled(&'a self, node: &'a SyntaxNode) -> Option<ArenaDoc<'a>> {
        if self.attr_store.is_format_disabled(node) {
            Some(self.convert_verbatim_untyped(node))
        } else {
            None
        }
    }

    /// For inner or lead nodes.
    fn convert_verbatim(&'a self, node: impl AstNode<'a>) -> ArenaDoc<'a> {
        self.convert_verbatim_untyped(node.to_untyped())
    }

    /// For inner or lead nodes.
    fn convert_verbatim_untyped(&'a self, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        let text = node.clone().into_text();
        if !text.has_linebreak() {
            return self.arena.text(text.to_string());
        }
        // When the text spans multiple lines, we should split it to ensure proper fitting.
        self.arena
            .intersperse(
                node.clone().into_text().lines().map(str::to_string),
                self.arena.hardline(),
            )
            .dedent_to_root()
    }

    /// For leaf only.
    fn convert_trivia(&'a self, node: impl AstNode<'a>) -> ArenaDoc<'a> {
        self.convert_trivia_untyped(node.to_untyped())
    }

    /// For leaf only.
    fn convert_trivia_untyped(&'a self, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        self.emit_source_text_untyped(node, node.text().as_str())
    }

    pub fn try_convert_with_mode(
        &'a self,
        node: &'a SyntaxNode,
        mode: Mode,
    ) -> Result<ArenaDoc<'a>, Error> {
        let ctx = Context::default().with_mode(mode);
        let doc = if let Some(markup) = node.cast() {
            self.convert_markup(ctx, markup)
        } else if let Some(code) = node.cast() {
            self.convert_code(ctx, code)
        } else if let Some(math) = node.cast() {
            self.convert_math(ctx, math)
        } else if let Some(expr) = node.cast() {
            self.convert_expr(ctx, expr)
        } else if let Some(pattern) = node.cast() {
            self.convert_pattern(ctx, pattern)
        } else {
            return Err(Error::SyntaxError);
        };
        Ok(doc)
    }

    pub fn convert_expr(&'a self, ctx: Context, expr: Expr<'a>) -> ArenaDoc<'a> {
        if let Some(res) = self.check_disabled(expr.to_untyped()) {
            return res;
        }
        self.convert_expr_impl(ctx, expr)
    }

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
            Expr::MathAlignPoint(map) => self.convert_trivia(map),
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

    fn convert_literal(&'a self, literal: &'a str) -> ArenaDoc<'a> {
        self.arena.text(literal)
    }
}
