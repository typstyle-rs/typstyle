//! Maps contextual prose boundaries back to the syntax nodes that layout emits.
//!
//! A break replaces a `Space` or `Linebreak`, splits a `Text` leaf, or starts the
//! next node on a new line after an unspaced CJK sentence ending. The last case is
//! attributed to the next node, or to the wrapper that owns it, so the break lands
//! before the wrapper's delimiters rather than inside them.
//!
//! Transparent inline markup shares the enclosing plan. Raw literals contribute
//! context but have no editable interior. Opaque expressions keep the existing
//! terminal-suffix fallback; their source code is never treated as prose.

use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use crate::{
    AttrStore,
    pretty::text::is_enum_marker,
    segmenter::{
        SentenceSegmenter, ends_with_cjk_terminator, has_terminal_suffix, is_terminal_punctuation,
    },
};

/// Characters that join words deliberately: no-break space, narrow no-break
/// space, and word joiner. A sentence boundary next to one must stay inline.
const PROTECTED_JOINS: [char; 3] = ['\u{00a0}', '\u{202f}', '\u{2060}'];

/// The layout actions that materialize sentence boundaries at one syntax node.
#[derive(Default)]
struct NodeBreaks {
    /// Start the next line before this node: an unspaced CJK sentence ending.
    before: bool,
    /// Replace this `Space` node with a hard line.
    space: bool,
    /// Keep this `Linebreak` on the previous sentence's line; the break moves to
    /// the following whitespace.
    linebreak: bool,
    /// Byte offsets inside this `Text` leaf where a new sentence starts.
    ///
    /// Two inline slots are enough: measured on the Typst docs corpus and large
    /// prose fixtures, 99.1% of entries hold at most two offsets (p99 = 2, max
    /// 7). A single slot would spill 11.7% of entries.
    text: SmallVec<[usize; 2]>,
}

/// A plan belongs to one borrowed syntax tree. Node addresses distinguish even
/// detached nodes with identical spans; they are only used as identity keys.
#[derive(Default)]
pub(in crate::pretty) struct MarkupSegmentation {
    /// Boundaries per node; nodes absent from the map carry none.
    nodes: FxHashMap<*const SyntaxNode, NodeBreaks>,
}

impl MarkupSegmentation {
    /// Builds the plan for one markup body. Nested transparent wrappers share it;
    /// opaque bodies start their own.
    pub(super) fn new(markup: Markup<'_>, attrs: &AttrStore) -> Self {
        let mut builder = Builder {
            segmenter: SentenceSegmenter::new(),
            text: String::new(),
            pieces: Default::default(),
            plan: Self::default(),
            attrs,
            leading_wrapper: None,
            pending_linebreak_break: false,
        };
        builder.markup(markup);
        builder.flush();
        builder.plan
    }

    /// The plan is keyed by node address because `SyntaxNode` has no identity
    /// beyond its position in the borrowed tree.
    fn breaks(&self, node: &SyntaxNode) -> Option<&NodeBreaks> {
        self.nodes.get(&std::ptr::from_ref(node))
    }

    /// Returns the entry for `node`, inserting a default one if it has none.
    fn breaks_mut(&mut self, node: &SyntaxNode) -> &mut NodeBreaks {
        self.nodes.entry(std::ptr::from_ref(node)).or_default()
    }

    /// Attributes a boundary in the synthesized run to the syntax node layout
    /// emits. `after_space` means the boundary consumed ASCII whitespace.
    fn record_boundary(&mut self, pieces: &[Piece<'_>], offset: usize, after_space: bool) {
        // ICU reports ASCII boundaries after the whitespace, so step one byte back
        // to land inside the owning Space piece. Unspaced boundaries point directly
        // at the node that starts the next sentence.
        let byte = if after_space { offset - 1 } else { offset };
        // Pieces are ordered by their end offset, so this finds the piece that
        // contains `byte` (or starts exactly at it).
        let index = pieces.partition_point(|piece| piece.end <= byte);
        let Some(piece) = pieces.get(index) else {
            return;
        };
        if piece.node.kind() == SyntaxKind::Space {
            // Whitespace boundaries replace the space.
            self.breaks_mut(piece.node).space = true;
        } else if offset == piece.start {
            // The next node starts a new sentence: break before it, or before the
            // wrapper that owns its first piece.
            let node = wrapper_target(piece, pieces, index);
            self.breaks_mut(node).before = true;
        } else if piece.node.kind() == SyntaxKind::Text {
            // The boundary is inside a text leaf; layout splits the leaf here.
            self.breaks_mut(piece.node).text.push(offset - piece.start);
        }
    }

    /// Returns whether this `Space` node becomes a hard line.
    pub(super) fn at_space(&self, node: &SyntaxNode) -> bool {
        self.breaks(node).is_some_and(|breaks| breaks.space)
    }

    /// Returns whether the next line starts before this node.
    pub(super) fn before(&self, node: &SyntaxNode) -> bool {
        self.breaks(node).is_some_and(|breaks| breaks.before)
    }

    /// Returns whether a forced linebreak ends the sentence before it, so `\` must stay
    /// attached to that sentence and the break is deferred to the following whitespace.
    pub(super) fn at_linebreak(&self, node: &SyntaxNode) -> bool {
        self.breaks(node).is_some_and(|breaks| breaks.linebreak)
    }

    /// Returns the byte offsets inside this `Text` leaf where a sentence starts.
    pub(super) fn in_text(&self, node: &SyntaxNode) -> &[usize] {
        self.breaks(node)
            .map_or(&[][..], |breaks| breaks.text.as_slice())
    }
}

/// A slice of the synthesized prose run.
struct Piece<'a> {
    /// Syntax node whose text occupies `start..end`.
    node: &'a SyntaxNode,
    /// Byte span in [`Builder::text`].
    start: usize,
    /// Exclusive end of the byte span in [`Builder::text`].
    end: usize,
    /// The outermost inline wrapper whose first content is this piece, if any.
    leading_wrapper: Option<&'a SyntaxNode>,
}

/// Accumulates one prose run and records where its sentence boundaries land in
/// the syntax tree.
struct Builder<'a, 'attrs> {
    /// ICU policy applied to the synthesized run.
    segmenter: SentenceSegmenter,
    /// Prose accumulated since the last flush, with markup delimiters removed.
    text: String,
    /// Maps byte spans in `text` back to the syntax nodes that produced them.
    ///
    /// Most markup bodies peak at a handful of pieces: on the Typst docs corpus
    /// and large prose fixtures, 92.8% stay within eight and 90% within seven.
    /// Inline storage keeps those bodies allocation-free.
    pieces: SmallVec<[Piece<'a>; 8]>,
    /// Boundaries attributed to nodes; layout consumes this.
    plan: MarkupSegmentation,
    /// Lookup for `@typstyle off` regions.
    attrs: &'attrs AttrStore,
    /// Outermost wrapper whose first piece has not been recorded yet.
    leading_wrapper: Option<&'a SyntaxNode>,
    /// Whether the next editable whitespace must carry a sentence break that was
    /// deferred across an attached `\`.
    pending_linebreak_break: bool,
}

impl<'a> Builder<'a, '_> {
    /// Appends prose and records its owning node. The first piece inside a wrapper
    /// takes the pending wrapper, so a boundary there can break before the
    /// delimiters.
    fn push(&mut self, node: &'a SyntaxNode, text: &str) {
        if text.is_empty() {
            return;
        }
        let start = self.text.len();
        self.text.push_str(text);
        self.record_piece(node, start);
    }

    /// Appends a decoded escape or shorthand character.
    fn push_char(&mut self, node: &'a SyntaxNode, c: char) {
        let start = self.text.len();
        self.text.push(c);
        self.record_piece(node, start);
    }

    /// Records the span that was just appended to `text`.
    fn record_piece(&mut self, node: &'a SyntaxNode, start: usize) {
        self.pieces.push(Piece {
            node,
            start,
            end: self.text.len(),
            leading_wrapper: self.leading_wrapper.take(),
        });
    }

    /// Enters a transparent wrapper. Its first piece remembers the outermost
    /// wrapper, so a break at that position lands before the delimiters.
    fn markup_wrapped(&mut self, body: Markup<'a>, node: &'a SyntaxNode) {
        let installed = self.leading_wrapper.is_none();
        if installed {
            self.leading_wrapper = Some(node);
        }
        self.markup(body);
        if installed {
            self.leading_wrapper = None;
        }
    }

    /// Walks one markup body in source order, extending the run through
    /// transparent nodes and flushing it at opaque or structural boundaries.
    fn markup(&mut self, markup: Markup<'a>) {
        for node in markup.to_untyped().children() {
            // Disabled source is opaque: it contributes no prose and must not be
            // analyzed as part of the surrounding sentence.
            if self.attrs.is_format_disabled(node) {
                self.flush();
                continue;
            }

            // `Expr::cast` skips whitespace, but space contributes prose context.
            // Soft source newlines must not become Unicode paragraph separators;
            // layout still owns their preservation.
            if node.kind() == SyntaxKind::Space {
                self.push(node, " ");
                if self.pending_linebreak_break {
                    // The sentence break deferred across an attached `\` lands here.
                    self.pending_linebreak_break = false;
                    self.plan.breaks_mut(node).space = true;
                }
                continue;
            }

            self.pending_linebreak_break = false;

            // Comments are the only remaining markup children without an expression.
            // A block comment is metadata; a line comment ends the prose run. Anything
            // else is treated as opaque source.
            let Some(expr) = node.cast::<Expr>() else {
                match node.kind() {
                    SyntaxKind::BlockComment => {}
                    SyntaxKind::LineComment => self.flush(),
                    _ => self.flush_opaque(node),
                }
                continue;
            };

            // Transparent nodes extend the current run; opaque nodes end it.
            match expr {
                Expr::Text(text) => self.push(node, text.get()),
                // Strong and Emph are transparent. The first piece inside remembers
                // the outermost wrapper so a break at that point lands before `*`/`_`
                // instead of splitting the delimiters.
                Expr::Strong(strong) => self.markup_wrapped(strong.body(), node),
                Expr::Emph(emph) => self.markup_wrapped(emph.body(), node),
                Expr::SmartQuote(_) => self.push(node, node.leaf_text()),
                Expr::Escape(escape) => self.push_char(node, escape.get()),
                Expr::Shorthand(shorthand) => self.push_char(node, shorthand.get()),
                // Inline raw contributes context but has no editable interior:
                // boundaries inside it are never applied.
                Expr::Raw(raw) if !raw.block() => {
                    let start = self.text.len();
                    for (index, line) in raw.lines().enumerate() {
                        if index > 0 {
                            self.text.push('\n');
                        }
                        self.text.push_str(line.get());
                    }
                    if self.text.len() > start {
                        self.record_piece(node, start);
                    }
                }
                // The equation is opaque; only a trailing punctuation atom is
                // evidence that the sentence ends at the math edge.
                Expr::Equation(equation) if !equation.block() => {
                    self.flush_opaque(node);
                    if let Some(last) = equation.body().to_untyped().children().last()
                        && last.kind() == SyntaxKind::MathText
                        && last.leaf_text().ends_with(is_terminal_punctuation)
                    {
                        self.push(node, last.leaf_text());
                    }
                }
                // Metadata stays in the source stream but contributes no prose and
                // does not interrupt the sentence.
                Expr::Label(_) => {}
                // A forced linebreak stays on the line of the sentence it terminates.
                // The break itself is emitted by the following whitespace.
                Expr::Linebreak(_) => self.flush_linebreak(node),
                // Structural boundaries end the run without the opaque fallback.
                Expr::Parbreak(_) => self.flush(),
                // Every other expression is opaque: its source is never prose, but a
                // terminal CJK ending in the preceding prose still starts a new line.
                _ => self.flush_opaque(node),
            }
        }
    }

    /// Segments the accumulated prose run without resetting builder state.
    fn segment_run(&mut self) {
        for offset in self.segmenter.boundaries(&self.text) {
            // A no-break space is semantic content, even when ICU proposes a
            // sentence boundary after it. Do not replace it with source layout.
            let (before, after) = self.text.split_at(offset);
            if has_protected_join(before) || cannot_start_line(after) {
                continue;
            }
            // Accept a boundary after editable whitespace, or an unspaced CJK
            // sentence ending. ASCII punctuation alone is not enough without a
            // whitespace position for layout to replace.
            let after_space = before.ends_with(|c: char| c.is_ascii_whitespace());
            if after_space
                || (!before.ends_with(char::is_whitespace)
                    && !after.starts_with(PROTECTED_JOINS)
                    && ends_with_cjk_terminator(before))
            {
                self.plan.record_boundary(&self.pieces, offset, after_space);
            }
        }
    }

    /// Records the boundaries of the accumulated prose run and resets run state.
    fn flush(&mut self) {
        if !self.text.is_empty() {
            self.segment_run();
            self.text.clear();
            self.pieces.clear();
        }
        self.leading_wrapper = None;
        // A run ending at a structural boundary cannot carry an attached `\` into the
        // next run; `flush_linebreak` re-arms this when a sentence ends before `\`.
        self.pending_linebreak_break = false;
    }

    /// Flushes the preceding prose run before an opaque node, applying the
    /// terminal-punctuation edge fallback when no interior prose follows.
    fn flush_opaque(&mut self, node: &'a SyntaxNode) {
        let target = self.leading_wrapper.take().unwrap_or(node);
        if !self.text.is_empty() {
            self.segment_run();
            if !has_protected_join(&self.text) {
                if self.text.ends_with(|c: char| c.is_ascii_whitespace()) {
                    if has_terminal_suffix(&self.text) {
                        // Opaque edge with a trailing space: the last space is the only
                        // position layout can turn into a break.
                        self.plan
                            .record_boundary(&self.pieces, self.text.len(), true);
                    }
                } else if !self.text.ends_with(char::is_whitespace)
                    && ends_with_cjk_terminator(&self.text)
                {
                    // Opaque edge with an unspaced CJK ending: start the opaque node on
                    // a new line. A protected join or trailing whitespace suppresses it.
                    self.plan.breaks_mut(target).before = true;
                }
            }
            self.text.clear();
            self.pieces.clear();
        }
        self.pending_linebreak_break = false;
    }

    /// Flushes the preceding prose run before a forced linebreak `\`.
    ///
    /// When the run ended a sentence, `\` stays attached to that sentence's line
    /// and the sentence break is deferred to the following editable whitespace.
    fn flush_linebreak(&mut self, node: &'a SyntaxNode) {
        let ended_sentence = !self.text.is_empty()
            && !has_protected_join(&self.text)
            && has_terminal_suffix(&self.text);
        self.flush();
        if ended_sentence {
            self.plan.breaks_mut(node).linebreak = true;
            self.pending_linebreak_break = true;
        }
    }
}

/// Returns the node that owns a break at `offset`, preferring the wrapper whose
/// first piece starts here. ICU can attach an unspaced CJK boundary after an
/// opening quote that begins the wrapper, so a preceding wrapped quote also
/// resolves to its wrapper.
fn wrapper_target<'a>(piece: &Piece<'a>, pieces: &[Piece<'a>], index: usize) -> &'a SyntaxNode {
    if let Some(wrapper) = piece.leading_wrapper {
        return wrapper;
    }
    if let Some(prev) = index.checked_sub(1).and_then(|index| pieces.get(index))
        && prev.end == piece.start
        && prev.node.kind() == SyntaxKind::SmartQuote
        && let Some(wrapper) = prev.leading_wrapper
    {
        return wrapper;
    }
    piece.node
}

/// Returns whether `text` ends at a [`PROTECTED_JOINS`] character, ignoring
/// trailing whitespace. Such a boundary must not become layout whitespace.
fn has_protected_join(text: &str) -> bool {
    text.chars()
        .rev()
        .take_while(|c| c.is_whitespace() || *c == '\u{2060}')
        .any(|c| PROTECTED_JOINS.contains(&c))
}

/// Returns whether `text` would read as line-sensitive markup at the start of a
/// line, such as a list marker or heading sign, so a break before it must stay
/// inline.
fn cannot_start_line(text: &str) -> bool {
    let text = text.trim_ascii_start();
    let ends_token = |rest: &str| rest.as_bytes().first().is_none_or(u8::is_ascii_whitespace);
    // Only inspect a marker-shaped prefix. The next whitespace can be much
    // farther away when the document has adjacent unspaced sentences.
    match text.as_bytes() {
        [b'+' | b'-' | b'/', ..] if ends_token(&text[1..]) => true,
        [b'=', ..] => ends_token(text.trim_start_matches('=')),
        [b'+' | b'0'..=b'9', ..] => {
            let digits = text.strip_prefix('+').unwrap_or(text);
            let Some(rest) = digits
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .strip_prefix('.')
            else {
                return false;
            };
            ends_token(rest) && is_enum_marker(&text[..text.len() - rest.len()])
        }
        _ => false,
    }
}
