use prettyless::Doc;
use smallvec::SmallVec;
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{
    Context, Mode, PrettyPrinter, layout::flow::FlowItem, prelude::*, text::is_enum_marker,
    util::is_comment_node,
};
use crate::{ext::StrExt, pretty::util::is_only_one_and};

#[derive(Debug, PartialEq, Eq)]
enum MarkupScope {
    /// The top-level markup.
    Document,
    /// Markup enclosed by `[]`.
    ContentBlock,
    /// Strong or Emph.
    Strong,
    /// ListItem, EnumItem, desc of TermItem. Spaces without linebreaks can be stripped.
    Item,
    /// Heading, term of TermItem. Like `Item`, but linebreaks are not allowed.
    InlineItem,
}

impl MarkupScope {
    fn can_trim(&self) -> bool {
        matches!(self, Self::Item | Self::InlineItem)
    }
}

impl<'a> PrettyPrinter<'a> {
    pub fn convert_markup(&'a self, ctx: Context, markup: Markup<'a>) -> ArenaDoc<'a> {
        self.convert_markup_impl(ctx, markup, MarkupScope::Document)
    }

    pub(super) fn convert_content_block(
        &'a self,
        ctx: Context,
        content_block: ContentBlock<'a>,
    ) -> ArenaDoc<'a> {
        let content =
            self.convert_markup_impl(ctx, content_block.body(), MarkupScope::ContentBlock);
        content.group().brackets()
    }

    pub(super) fn convert_strong(&'a self, ctx: Context, strong: Strong<'a>) -> ArenaDoc<'a> {
        let body = self.convert_markup_impl(ctx, strong.body(), MarkupScope::Strong);
        body.enclose("*", "*")
    }

    pub(super) fn convert_emph(&'a self, ctx: Context, emph: Emph<'a>) -> ArenaDoc<'a> {
        let body = self.convert_markup_impl(ctx, emph.body(), MarkupScope::Strong);
        body.enclose("_", "_")
    }

    pub(super) fn convert_raw(&'a self, ctx: Context, raw: Raw<'a>) -> ArenaDoc<'a> {
        // no format multiline single backtick raw block
        if !raw.block() && raw.lines().nth(1).is_some() {
            return self.convert_verbatim(raw);
        }

        let mut doc = self.arena.nil();
        for child in raw.to_untyped().children() {
            if let Some(delim) = child.cast::<RawDelim>() {
                doc += self.convert_trivia(delim);
            } else if let Some(lang) = child.cast::<RawLang>() {
                doc += self.convert_trivia(lang);
            } else if let Some(text) = child.cast::<Text>() {
                doc += self.convert_text(text);
            } else if child.kind() == SyntaxKind::RawTrimmed {
                doc += self.convert_space_untyped(ctx, child);
            }
        }
        doc
    }

    pub(super) fn convert_ref(&'a self, ctx: Context, reference: Ref<'a>) -> ArenaDoc<'a> {
        let mut doc = self.arena.text("@") + self.arena.text(reference.target());
        if let Some(supplement) = reference.supplement() {
            doc += self.convert_content_block(ctx, supplement);
        }
        doc
    }

    pub(super) fn convert_heading(&'a self, ctx: Context, heading: Heading<'a>) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, heading.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::HeadingMarker {
                FlowItem::spaced(self.emit_source_text_untyped(child, child.text().as_str()))
            } else if let Some(markup) = child.cast::<Markup>() {
                if !child.is_empty() {
                    FlowItem::spaced(self.convert_markup_impl(ctx, markup, MarkupScope::InlineItem))
                } else {
                    FlowItem::none()
                }
            } else {
                FlowItem::none()
            }
        })
    }

    pub(super) fn convert_list_item(
        &'a self,
        ctx: Context,
        list_item: ListItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_list_item_like(ctx, list_item.to_untyped())
    }

    pub(super) fn convert_enum_item(
        &'a self,
        ctx: Context,
        enum_item: EnumItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_list_item_like(ctx, enum_item.to_untyped())
    }

    pub(super) fn convert_term_item(
        &'a self,
        ctx: Context,
        term_item: TermItem<'a>,
    ) -> ArenaDoc<'a> {
        let node = term_item.to_untyped();
        let mut seen_term = false;
        let body = self.convert_flow_like(ctx, node, |ctx, child, _| match child.kind() {
            SyntaxKind::TermMarker => {
                FlowItem::spaced(self.emit_source_text_untyped(child, child.text().as_str()))
            }
            SyntaxKind::Colon => {
                seen_term = true;
                FlowItem::tight_spaced(self.emit_source_text_untyped(child, child.text().as_str()))
            }
            SyntaxKind::Space if child.text().has_linebreak() => {
                FlowItem::tight(self.arena.hardline())
            }
            SyntaxKind::Parbreak => FlowItem::tight(
                self.arena
                    .hardline()
                    .repeat(child.text().count_linebreaks()),
            ),
            SyntaxKind::Markup => {
                if !seen_term || !child.is_empty() {
                    // empty markup is ignored here
                    FlowItem::spaced(self.convert_markup_impl(
                        ctx,
                        child.cast().expect("markup"),
                        if !seen_term {
                            MarkupScope::InlineItem
                        } else {
                            MarkupScope::Item
                        },
                    ))
                } else {
                    FlowItem::none()
                }
            }
            _ => FlowItem::none(),
        });
        self.indent(body)
    }

    fn convert_list_item_like(&'a self, ctx: Context, item: &'a SyntaxNode) -> ArenaDoc<'a> {
        let body = self.convert_flow_like(ctx, item, |ctx, child, _| match child.kind() {
            SyntaxKind::ListMarker | SyntaxKind::EnumMarker | SyntaxKind::TermMarker => {
                FlowItem::spaced(self.emit_source_text_untyped(child, child.text().as_str()))
            }
            SyntaxKind::Space if child.text().has_linebreak() => {
                FlowItem::tight(self.arena.hardline())
            }
            SyntaxKind::Parbreak => FlowItem::tight(
                self.arena
                    .hardline()
                    .repeat(child.text().count_linebreaks()),
            ),
            SyntaxKind::Markup if !child.is_empty() => {
                // empty markup is ignored here
                FlowItem::spaced(self.convert_markup_impl(
                    ctx,
                    child.cast().expect("markup"),
                    MarkupScope::Item,
                ))
            }
            _ => FlowItem::none(),
        });
        self.indent(body)
    }

    fn convert_markup_impl(
        &'a self,
        ctx: Context,
        markup: Markup<'a>,
        scope: MarkupScope,
    ) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::Markup);

        // If the markup only contains one space, simply convert it.
        let children = markup.to_untyped().children().as_slice();
        if children.len() == 1
            && let Some(space) = children[0].cast::<Space>()
        {
            return self.convert_space(ctx, space);
        }

        let repr = collect_markup_repr(markup);
        let body = if self.config.wrap_text && scope != MarkupScope::InlineItem {
            self.convert_markup_body_reflow(ctx, &repr)
        } else {
            self.convert_markup_body(ctx, &repr)
        };

        // Add line or space (if any) to both sides.
        // Only turn space into, not the other way around.
        let get_delim = |bound: Boundary| {
            if scope == MarkupScope::Document || scope.can_trim() {
                // should not add extra lines to the document
                return if bound == Boundary::Break {
                    self.arena.hardline()
                } else {
                    self.arena.nil()
                };
            }
            match bound {
                Boundary::Nil => self.arena.nil(),
                Boundary::NilOrBreak => {
                    if (scope.can_trim() || ctx.break_suppressed) && !self.config.wrap_text {
                        self.arena.nil()
                    } else {
                        self.arena.line_()
                    }
                }
                Boundary::WeakNilOrBreak => {
                    if self.config.wrap_text {
                        self.arena.line_()
                    } else {
                        self.arena.nil()
                    }
                }
                Boundary::Space(n) => {
                    if scope.can_trim() {
                        // the space can be safely eaten
                        self.arena.nil()
                    } else if self.config.wrap_text {
                        self.arena.line()
                    } else if self.config.collapse_markup_spaces {
                        self.arena.space()
                    } else {
                        self.arena.spaces(n)
                    }
                }
                Boundary::Break | Boundary::WeakBreak => self.arena.hardline(),
            }
        };

        let open = get_delim(repr.start_bound);
        let close = get_delim(repr.end_bound);
        // Do not indent (compact), if the opening will not break.
        let needs_indent = matches!(scope, MarkupScope::ContentBlock)
            && !(matches!(*open, Doc::Nil | Doc::Text(_))
                && contains_exactly_one_primary_expr(markup));
        let body_with_before = open + body;
        let body_with_before = if needs_indent {
            self.indent(body_with_before)
        } else {
            // Use compact layout.
            body_with_before
        };
        (body_with_before + close).group()
    }

    fn convert_markup_body(&'a self, ctx: Context, repr: &MarkupRepr<'a>) -> ArenaDoc<'a> {
        let mut doc = self.arena.nil();
        for &MarkupLine {
            ref nodes,
            breaks,
            mixed_text,
        } in repr.lines.iter()
        {
            for node in nodes.iter() {
                doc += if node.kind() == SyntaxKind::Space {
                    self.convert_space_untyped(ctx, node)
                } else if let Some(text) = node.cast::<Text>() {
                    self.convert_text(text)
                } else if let Some(expr) = node.cast::<Expr>() {
                    let ctx = if mixed_text {
                        ctx.suppress_breaks()
                    } else {
                        ctx
                    };
                    self.convert_expr(ctx, expr)
                } else if is_comment_node(node) {
                    self.convert_comment(ctx, node)
                } else {
                    // can be Hash, Semicolon, Shebang
                    self.convert_trivia_untyped(node)
                };
            }
            if breaks > 0 {
                doc += self.arena.hardline().repeat(breaks);
            }
        }
        doc
    }

    /// With text-wrapping enabled, spaces may turn to linebreaks, and linebreaks may turn to spaces, if safe.
    fn convert_markup_body_reflow(&'a self, ctx: Context, repr: &MarkupRepr<'a>) -> ArenaDoc<'a> {
        /// For NOT space -> soft-line: \
        /// Ensure they are not misinterpreted as markup markers after reflow.
        ///
        /// Besides, reflowing labels to the next line is not desired.
        fn cannot_break_before(node: &&SyntaxNode) -> bool {
            let text = node.text();
            matches!(text.as_str(), "=" | "+" | "-" | "/")
                || matches!(node.kind(), SyntaxKind::Label)
                || is_enum_marker(text)
        }

        /// For space -> hard-line: \
        /// Prefers block equations exclusive to a single line.
        fn prefer_exclusive(node: &&SyntaxNode) -> bool {
            is_block_equation(node) || is_block_raw(node)
        }

        /// For NOT hard-line -> soft-line: \
        /// Should always break after block elements or line comments.
        fn should_break_after(node: &SyntaxNode) -> bool {
            matches!(
                node.kind(),
                SyntaxKind::Heading
                    | SyntaxKind::ListItem
                    | SyntaxKind::EnumItem
                    | SyntaxKind::TermItem
                    | SyntaxKind::LineComment
            )
        }

        /// For NOT hard-line -> soft-line: \
        /// Breaking after them is visually better.
        fn preserve_break_after(node: &SyntaxNode) -> bool {
            matches!(
                node.kind(),
                SyntaxKind::BlockComment
                    | SyntaxKind::Linebreak
                    | SyntaxKind::Label
                    | SyntaxKind::CodeBlock
                    | SyntaxKind::ContentBlock
                    | SyntaxKind::Conditional
                    | SyntaxKind::WhileLoop
                    | SyntaxKind::ForLoop
                    | SyntaxKind::Contextual
            ) || is_block_equation(node)
                || is_block_raw(node)
        }

        /// For NOT hard-line -> soft-line: \
        /// Keeps the line exclusive (prevents soft breaks) when:
        /// - It contains only one non-text node, or
        /// - It contains exactly two nodes where the first is a Hash, such as `#figure()`.
        fn preserve_exclusive(line: &MarkupLine) -> bool {
            let nodes = &line.nodes;
            let len = nodes.len();
            len == 1 && nodes[0].kind() != SyntaxKind::Text
                || len == 2 && nodes[0].kind() == SyntaxKind::Hash
                || len > 0 && prefer_exclusive(&nodes[0])
        }

        let mut doc = self.arena.nil();
        for (i, line) in repr.lines.iter().enumerate() {
            let &MarkupLine {
                ref nodes, breaks, ..
            } = line;
            for (j, node) in nodes.iter().enumerate() {
                doc += if node.kind() == SyntaxKind::Space {
                    if nodes.get(j + 1).is_some_and(cannot_break_before) {
                        self.arena.space()
                    } else if nodes.get(j + 1).is_some_and(prefer_exclusive)
                        || nodes.get(j - 1).is_some_and(prefer_exclusive)
                    {
                        self.arena.hardline()
                    } else {
                        self.arena.softline()
                    }
                } else if let Some(text) = node.cast::<Text>() {
                    self.convert_text_wrapped(text)
                } else if let Some(expr) = node.cast::<Expr>() {
                    self.convert_expr(ctx, expr)
                } else if is_comment_node(node) {
                    self.convert_comment(ctx, node)
                } else {
                    // can be Hash, Semicolon, Shebang
                    self.convert_trivia_untyped(node)
                };
            }
            // Should not eat trailing parbreaks.
            if breaks == 1
                && i + 1 != repr.lines.len()
                && !nodes
                    .last()
                    .is_some_and(|last| should_break_after(last) || preserve_break_after(last))
                && !preserve_exclusive(line)
                && !preserve_exclusive(&repr.lines[i + 1])
            {
                doc += self.arena.softline();
            } else if breaks > 0 {
                doc += self.arena.hardline().repeat(breaks);
            }
        }
        doc
    }
}

#[derive(Default)]
struct MarkupLine<'a> {
    nodes: SmallVec<[&'a SyntaxNode; 4]>,
    breaks: usize,
    mixed_text: bool,
}

struct MarkupRepr<'a> {
    lines: Vec<MarkupLine<'a>>,
    start_bound: Boundary,
    end_bound: Boundary,
}

/// Markup boundary, deciding whether can break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Boundary {
    /// Should add no blank.
    Nil,
    /// Beside blocky elements. Can turn to a linebreak when multiline.
    NilOrBreak,
    /// Can turn to a linebreak if not in document scope and text-wrapping enabled,
    /// as there are already spaces after comments.
    WeakNilOrBreak,
    /// n spaces.
    Space(usize),
    /// Always breaks.
    Break,
    /// Always breaks if not in document scope.
    WeakBreak,
}

impl Boundary {
    pub fn from_space(space: &str) -> Self {
        if space.has_linebreak() {
            Self::Break
        } else {
            Self::Space(space.len())
        }
    }

    pub fn strip_space(self) -> Self {
        match self {
            Self::Space(_) => Self::NilOrBreak,
            _ => self,
        }
    }
}

// Break markup into lines, split by stmt, parbreak, newline, multiline raw,
// equation if a line contains text, it will be skipped by the formatter
// to keep the original format.
fn collect_markup_repr(markup: Markup<'_>) -> MarkupRepr<'_> {
    /// A subset of "blocky" elements that we cannot safely handle currently.
    /// By default show rule, these elements seem to have weak spaces on both sides.
    /// But this behavior can be changed by wrapping them in a box.
    fn is_special_block_elem(it: &SyntaxNode) -> bool {
        matches!(
            it.kind(),
            SyntaxKind::ListItem | SyntaxKind::EnumItem | SyntaxKind::TermItem
        )
    }

    let mut repr = MarkupRepr {
        lines: vec![],
        start_bound: Boundary::Nil,
        end_bound: Boundary::Nil,
    };
    let mut current_line = MarkupLine::default();
    for node in markup.to_untyped().children() {
        let break_line = match node.kind() {
            SyntaxKind::Parbreak => {
                current_line.breaks = node.text().count_linebreaks(); // This is >= 2
                true
            }
            SyntaxKind::Space if current_line.nodes.is_empty() => {
                // Due to the logic of line-slitting, it must also be the first node in the markup.
                debug_assert!(repr.lines.is_empty());
                repr.start_bound = Boundary::from_space(node.text());
                continue;
            }
            SyntaxKind::Space if node.text().has_linebreak() => {
                current_line.breaks = 1; // Must only one
                true
            }
            _ => {
                if matches!(
                    node.kind(),
                    SyntaxKind::Text | SyntaxKind::Strong | SyntaxKind::Emph | SyntaxKind::Raw
                ) {
                    current_line.mixed_text = true;
                }
                if current_line.nodes.is_empty() && is_special_block_elem(node) {
                    repr.start_bound = repr.start_bound.strip_space();
                }
                current_line.nodes.push(node);
                false
            }
        };
        if break_line {
            repr.lines.push(current_line);
            current_line = MarkupLine::default();
        }
    }
    if !current_line.nodes.is_empty() {
        repr.lines.push(current_line);
    }

    // Remove trailing spaces
    if let Some(last_line) = repr.lines.last_mut() {
        if last_line.breaks > 0 {
            last_line.breaks -= 1;
            repr.end_bound = Boundary::Break;
        }
        while let Some(last) = last_line.nodes.last() {
            if last.kind() == SyntaxKind::Space {
                repr.end_bound = Boundary::from_space(last.text());
                last_line.nodes.pop();
            } else {
                if is_special_block_elem(last) {
                    repr.end_bound = repr.end_bound.strip_space();
                }
                break;
            }
        }
    }

    // Check boundary through comments
    if repr.start_bound == Boundary::Nil
        && let Some(first_line) = repr.lines.first()
    {
        match first_line.nodes.iter().find(|it| !is_comment_node(it)) {
            Some(it) if is_special_block_elem(it) => {
                repr.start_bound = Boundary::NilOrBreak;
            }
            Some(it) if it.kind() == SyntaxKind::Space => {
                repr.start_bound = Boundary::WeakNilOrBreak;
            }
            None if !first_line.nodes.is_empty() => repr.start_bound = Boundary::WeakBreak,
            _ => {}
        }
    }
    if repr.end_bound == Boundary::Nil
        && let Some(last_line) = repr.lines.last()
    {
        match last_line.nodes.iter().rfind(|it| !is_comment_node(it)) {
            Some(it) if is_special_block_elem(it) => {
                repr.end_bound = Boundary::NilOrBreak;
            }
            Some(it) if it.kind() == SyntaxKind::Space => {
                repr.end_bound = Boundary::WeakNilOrBreak;
            }
            None if !last_line.nodes.is_empty() => repr.end_bound = Boundary::WeakBreak,
            _ => {}
        }
    }

    repr
}

fn is_block_equation(it: &SyntaxNode) -> bool {
    it.cast::<Equation>()
        .is_some_and(|equation| equation.block())
}

fn is_block_raw(it: &SyntaxNode) -> bool {
    it.cast::<Raw>().is_some_and(|raw| raw.block())
}

/// Returns true if the given markup contains exactly one primary (non-text, non-block) expression,
/// ignoring spaces, linebreaks, and labels, and no linebreak or parbreak presented.
fn contains_exactly_one_primary_expr(markup: Markup) -> bool {
    // Fast fail: if any linebreak or parbreak is present, not a single primary expr.
    if markup.exprs().any(|expr| {
        matches!(expr, Expr::Space(_)) && expr.to_untyped().text().has_linebreak()
            || matches!(expr, Expr::Parbreak(_))
    }) {
        return false;
    }
    is_only_one_and(
        markup
            .exprs()
            .filter(|it| !matches!(it, Expr::Space(_) | Expr::Linebreak(_) | Expr::Label(_))),
        |it| {
            // Blocky expressions may produce new breaks.
            // Other markup expressions are safe, as they must span only one line,
            // or can be covered in boundary check.
            !matches!(it, Expr::Text(_))
        },
    )
}
