//! Converts markup constructs: content blocks, strong and emph, raw blocks, references,
//! headings, and list, enum, and term items.

mod wrapping;

use Option::None;
use prettyless::Doc;
use smallvec::SmallVec;
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{
    Context, Mode, PrettyPrinter, layout::flow::FlowItem, prelude::*, util::is_comment_node,
};
use crate::{WrapMode, ext::StrExt, pretty::util::is_only_one_and};

/// Distinguishes how a markup body is laid out and how whitespace at its boundaries
/// may be trimmed or turned into line breaks.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MarkupScope {
    /// The top-level markup.
    Document,
    /// A selection within markup, retaining its own boundaries.
    Fragment,
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
    /// Returns whether leading and trailing whitespace of the markup may be stripped.
    fn can_trim(&self) -> bool {
        matches!(self, Self::Item | Self::InlineItem)
    }
}

impl<'a> PrettyPrinter<'a> {
    /// Converts top-level markup, forcing a trailing line break at the document end.
    pub fn convert_markup(&'a self, ctx: Context, markup: Markup<'a>) -> ArenaDoc<'a> {
        self.convert_markup_with_scope(ctx, markup, MarkupScope::Document)
    }

    /// Converts a content block `[...]` into a grouped, bracketed block.
    pub(super) fn convert_content_block(
        &'a self,
        ctx: Context,
        content_block: ContentBlock<'a>,
    ) -> ArenaDoc<'a> {
        let content =
            self.convert_markup_with_scope(ctx, content_block.body(), MarkupScope::ContentBlock);
        content.group().brackets()
    }

    /// Converts strong emphasis `*...*`.
    pub(super) fn convert_strong(&'a self, ctx: Context, strong: Strong<'a>) -> ArenaDoc<'a> {
        let body = self.convert_markup_with_scope(ctx, strong.body(), MarkupScope::Strong);
        body.enclose("*", "*")
    }

    /// Converts emphasis `_..._`.
    pub(super) fn convert_emph(&'a self, ctx: Context, emph: Emph<'a>) -> ArenaDoc<'a> {
        let body = self.convert_markup_with_scope(ctx, emph.body(), MarkupScope::Strong);
        body.enclose("_", "_")
    }

    /// Converts a raw block or inline raw span.
    ///
    /// Multiline spans delimited by a single backtick cannot be reformatted, so they are
    /// reproduced verbatim.
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

    /// Converts a reference `@target` with its optional supplement in brackets.
    pub(super) fn convert_ref(&'a self, ctx: Context, reference: Ref<'a>) -> ArenaDoc<'a> {
        let mut doc = self.arena.text("@") + self.arena.text(reference.target());
        if let Some(supplement) = reference.supplement() {
            doc += self.convert_content_block(ctx, supplement);
        }
        doc
    }

    /// Converts a heading, emitting its marker followed by the title as an inline item
    /// ([`MarkupScope::InlineItem`]).
    pub(super) fn convert_heading(&'a self, ctx: Context, heading: Heading<'a>) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, heading.to_untyped(), |ctx, child, _| {
            if child.kind() == SyntaxKind::HeadingMarker {
                FlowItem::spaced(self.arena.text(child.leaf_text().as_str()))
            } else if let Some(markup) = child.cast::<Markup>() {
                if !child.is_empty() {
                    FlowItem::spaced(self.convert_markup_with_scope(
                        ctx,
                        markup,
                        MarkupScope::InlineItem,
                    ))
                } else {
                    FlowItem::none()
                }
            } else {
                FlowItem::none()
            }
        })
    }

    /// Converts a list item `- ...`.
    pub(super) fn convert_list_item(
        &'a self,
        ctx: Context,
        list_item: ListItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_list_item_like(ctx, list_item.to_untyped())
    }

    /// Converts an enum item `1. ...`.
    pub(super) fn convert_enum_item(
        &'a self,
        ctx: Context,
        enum_item: EnumItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_list_item_like(ctx, enum_item.to_untyped())
    }

    /// Converts a term item `/ term: description`.
    ///
    /// The term is converted in [`MarkupScope::InlineItem`] scope and the description in
    /// [`MarkupScope::Item`] scope, preserving line breaks inside the description.
    pub(super) fn convert_term_item(
        &'a self,
        ctx: Context,
        term_item: TermItem<'a>,
    ) -> ArenaDoc<'a> {
        let node = term_item.to_untyped();
        let mut seen_term = false;
        let body = self.convert_flow_like(ctx, node, |ctx, child, _| match child.kind() {
            SyntaxKind::TermMarker => FlowItem::spaced(self.arena.text(child.leaf_text().as_str())),
            SyntaxKind::Colon => {
                seen_term = true;
                FlowItem::tight_spaced(self.arena.text(child.leaf_text().as_str()))
            }
            SyntaxKind::Space if child.leaf_text().has_linebreak() => {
                FlowItem::tight(self.arena.hard_line())
            }
            SyntaxKind::Parbreak => FlowItem::tight(
                self.arena
                    .hard_line()
                    .repeat(child.leaf_text().count_linebreaks()),
            ),
            SyntaxKind::Markup => {
                if !seen_term || !child.is_empty() {
                    // empty markup is ignored here
                    FlowItem::spaced(self.convert_markup_with_scope(
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

    /// Converts the body shared by list, enum, and term items: the marker followed by
    /// the content in [`MarkupScope::Item`] scope, indented one level.
    fn convert_list_item_like(&'a self, ctx: Context, item: &'a SyntaxNode) -> ArenaDoc<'a> {
        let body = self.convert_flow_like(ctx, item, |ctx, child, _| match child.kind() {
            SyntaxKind::ListMarker | SyntaxKind::EnumMarker | SyntaxKind::TermMarker => {
                FlowItem::spaced(self.arena.text(child.leaf_text().as_str()))
            }
            SyntaxKind::Space if child.leaf_text().has_linebreak() => {
                FlowItem::tight(self.arena.hard_line())
            }
            SyntaxKind::Parbreak => FlowItem::tight(
                self.arena
                    .hard_line()
                    .repeat(child.leaf_text().count_linebreaks()),
            ),
            SyntaxKind::Markup if !child.is_empty() => {
                // empty markup is ignored here
                FlowItem::spaced(self.convert_markup_with_scope(
                    ctx,
                    child.cast().expect("markup"),
                    MarkupScope::Item,
                ))
            }
            _ => FlowItem::none(),
        });
        self.indent(body)
    }

    /// Converts a markup node, honoring the given [`MarkupScope`].
    ///
    /// The markup is split into lines by [`collect_markup_repr`], the body is converted
    /// according to [`crate::Config::wrap_mode`], and the [`Boundary`] recorded at each
    /// end is materialized into spacing or line breaks.
    pub(super) fn convert_markup_with_scope(
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

        let mut repr = collect_markup_repr(markup);
        if scope == MarkupScope::Document {
            // In document scope, we should insert a trailing linebreak if not present.
            repr.end_bound = Boundary::Break;
        }
        let body = if scope == MarkupScope::InlineItem {
            self.convert_markup_body(ctx, &repr)
        } else {
            match self.config.wrap_mode {
                WrapMode::None => self.convert_markup_body(ctx, &repr),
                WrapMode::Sentence => self.convert_markup_body_sentence_per_line(ctx, &repr),
                WrapMode::Fill => self.convert_markup_body_wrapped(ctx, &repr, false),
                WrapMode::FillSentence => self.convert_markup_body_wrapped(ctx, &repr, true),
            }
        };

        // Add line or space (if any) to both sides.
        // Only turn space into, not the other way around.
        let get_delim = |bound: Boundary| {
            if scope == MarkupScope::Fragment
                && matches!(
                    bound,
                    Boundary::NilOrBreak | Boundary::WeakNilOrBreak | Boundary::WeakBreak
                )
            {
                // These boundaries are inferred from surrounding markup. A partial edit must
                // not materialize layout that lies outside the selected source range.
                return self.arena.nil();
            }
            if scope == MarkupScope::Document || scope.can_trim() {
                // should not add extra lines to the document
                return if bound == Boundary::Break {
                    self.arena.hard_line()
                } else {
                    self.arena.nil()
                };
            }
            match bound {
                Boundary::Nil => self.arena.nil(),
                Boundary::NilOrBreak => {
                    if (scope.can_trim() || ctx.break_suppressed)
                        && self.config.wrap_mode == WrapMode::None
                    {
                        self.arena.nil()
                    } else {
                        self.arena.line_()
                    }
                }
                Boundary::WeakNilOrBreak => {
                    if self.config.wrap_mode != WrapMode::None {
                        self.arena.line_()
                    } else {
                        self.arena.nil()
                    }
                }
                Boundary::Space(n) => {
                    if scope.can_trim() {
                        // the space can be safely eaten
                        self.arena.nil()
                    } else if self.config.wrap_mode != WrapMode::None {
                        self.arena.line()
                    } else if self.config.collapse_markup_spaces {
                        self.arena.space()
                    } else {
                        self.arena.spaces(n)
                    }
                }
                Boundary::Break | Boundary::WeakBreak => self.arena.hard_line(),
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

    /// Converts the markup lines without wrapping, preserving spaces and line breaks.
    ///
    /// Expressions on a line that mixes text with other markup are converted with breaks
    /// suppressed, so their inline layout stays as written.
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
                doc += self.arena.hard_line().repeat(breaks);
            }
        }
        doc
    }
}

/// A single line of markup, as split by [`collect_markup_repr`].
#[derive(Default)]
struct MarkupLine<'a> {
    /// The syntax nodes of this line.
    nodes: SmallVec<[&'a SyntaxNode; 4]>,
    /// The number of line breaks that follow this line.
    breaks: usize,
    /// Whether the line contains text alongside other markup, so breaks within it are
    /// suppressed.
    mixed_text: bool,
}

/// The line structure of a markup body, with the whitespace boundary at each end.
struct MarkupRepr<'a> {
    /// The lines of the markup, split at parbreaks and line breaks.
    lines: Vec<MarkupLine<'a>>,
    /// The whitespace before the first node of the markup.
    start_bound: Boundary,
    /// The whitespace after the last node of the markup.
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
    /// Returns the boundary described by a run of source whitespace: [`Boundary::Break`]
    /// if it contains a linebreak, [`Boundary::Space`] otherwise.
    pub fn from_space(space: &str) -> Self {
        if space.has_linebreak() {
            Self::Break
        } else {
            Self::Space(space.len())
        }
    }

    /// Turns a [`Boundary::Space`] into [`Boundary::NilOrBreak`], leaving other
    /// boundaries unchanged.
    pub fn strip_space(self) -> Self {
        match self {
            Self::Space(_) => Self::NilOrBreak,
            _ => self,
        }
    }
}

/// Splits the markup into lines at parbreaks and line breaks, recording the whitespace
/// boundary at each end (peeking through comments) and which lines mix text with other
/// markup, so the formatter can keep their original layout instead of re-flowing them.
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
                current_line.breaks = node.leaf_text().count_linebreaks(); // This is >= 2
                true
            }
            SyntaxKind::Space if current_line.nodes.is_empty() => {
                // Due to the logic of line-slitting, it must also be the first node in the markup.
                debug_assert!(repr.lines.is_empty());
                repr.start_bound = Boundary::from_space(node.leaf_text());
                continue;
            }
            SyntaxKind::Space if node.leaf_text().has_linebreak() => {
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
                repr.end_bound = Boundary::from_space(last.leaf_text());
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

/// Returns true if the given markup contains exactly one primary (non-text, non-block) expression,
/// ignoring spaces, linebreaks, and labels, and no linebreak or parbreak presented.
fn contains_exactly_one_primary_expr(markup: Markup) -> bool {
    // Fast fail: if any linebreak or parbreak is present, not a single primary expr.
    if markup.exprs().any(|expr| {
        matches!(expr, Expr::Space(_)) && expr.to_untyped().leaf_text().has_linebreak()
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
