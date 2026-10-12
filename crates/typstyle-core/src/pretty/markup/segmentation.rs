//! Maps contextual prose boundaries back to the syntax nodes that layout emits.
//!
//! Transparent inline markup shares the enclosing plan. Raw literals contribute
//! context but have no editable interior. Opaque expressions keep the existing
//! terminal-suffix fallback; their source code is never treated as prose.

use rustc_hash::FxHashMap;
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use crate::{
    AttrStore,
    pretty::text::is_line_sensitive_markup_marker,
    segmenter::{SentenceSegmenter, ends_with_cjk_terminator, has_terminal_suffix},
};

const PROTECTED_JOINS: [char; 3] = ['\u{00a0}', '\u{202f}', '\u{2060}'];

#[derive(Default)]
struct NodeBreaks {
    before: bool,
    space: bool,
    linebreak: bool,
    text: Vec<usize>,
}

/// A plan belongs to one borrowed syntax tree. Node addresses distinguish even
/// detached nodes with identical spans; they are only used as identity keys.
#[derive(Default)]
pub(super) struct MarkupSegmentation {
    nodes: FxHashMap<*const SyntaxNode, NodeBreaks>,
}

impl MarkupSegmentation {
    pub(super) fn new(markup: Markup<'_>, attrs: &AttrStore) -> Self {
        let mut builder = Builder {
            segmenter: SentenceSegmenter::new(),
            text: String::new(),
            pieces: Vec::new(),
            plan: Self::default(),
            attrs,
            pending_linebreak_break: false,
        };
        builder.markup(markup, None);
        builder.flush(None);
        builder.plan
    }

    fn breaks(&self, node: &SyntaxNode) -> Option<&NodeBreaks> {
        self.nodes.get(&std::ptr::from_ref(node))
    }

    fn breaks_mut(&mut self, node: &SyntaxNode) -> &mut NodeBreaks {
        self.nodes.entry(std::ptr::from_ref(node)).or_default()
    }

    fn record_boundary(&mut self, pieces: &[Piece<'_>], offset: usize, after_space: bool) {
        // Replace the whitespace before the boundary, or insert before the next
        // text when there is no gap. This keeps closing delimiters on their line.
        let byte = if after_space { offset - 1 } else { offset };
        let index = pieces.partition_point(|piece| piece.end <= byte);
        let Some(piece) = pieces.get(index) else {
            return;
        };
        match piece.node.kind() {
            SyntaxKind::Space if after_space => self.breaks_mut(piece.node).space = true,
            SyntaxKind::Text if offset == piece.start => {
                let node = piece.leading_wrapper.unwrap_or(piece.node);
                self.breaks_mut(node).before = true;
            }
            SyntaxKind::Text => self.breaks_mut(piece.node).text.push(offset - piece.start),
            _ => {}
        }
    }

    pub(super) fn at_space(&self, node: &SyntaxNode) -> bool {
        self.breaks(node).is_some_and(|breaks| breaks.space)
    }

    pub(super) fn before(&self, node: &SyntaxNode) -> bool {
        self.breaks(node).is_some_and(|breaks| breaks.before)
    }

    /// Returns whether a forced linebreak ends the sentence before it, so `\` must stay
    /// attached to that sentence and the break is deferred to the following whitespace.
    pub(super) fn at_linebreak(&self, node: &SyntaxNode) -> bool {
        self.breaks(node).is_some_and(|breaks| breaks.linebreak)
    }

    pub(super) fn in_text(&self, node: &SyntaxNode) -> &[usize] {
        self.breaks(node).map_or(&[], |breaks| &breaks.text)
    }
}

struct Piece<'a> {
    node: &'a SyntaxNode,
    start: usize,
    end: usize,
    // At the start of emphasis, break before its opening delimiter.
    leading_wrapper: Option<&'a SyntaxNode>,
}

struct Builder<'a, 'attrs> {
    segmenter: SentenceSegmenter,
    text: String,
    pieces: Vec<Piece<'a>>,
    plan: MarkupSegmentation,
    attrs: &'attrs AttrStore,
    /// Whether the next editable whitespace must carry a sentence break that was
    /// deferred across an attached `\`.
    pending_linebreak_break: bool,
}

impl<'a> Builder<'a, '_> {
    fn push(&mut self, node: &'a SyntaxNode, text: &str) -> Option<&mut Piece<'a>> {
        if text.is_empty() {
            return None;
        }
        let start = self.text.len();
        self.text.push_str(text);
        self.pieces.push(Piece {
            node,
            start,
            end: self.text.len(),
            leading_wrapper: None,
        });
        self.pieces.last_mut()
    }

    fn markup(&mut self, markup: Markup<'a>, mut leading_wrapper: Option<&'a SyntaxNode>) {
        for node in markup.to_untyped().children() {
            let leading_wrapper = leading_wrapper.take();
            if self.attrs.is_format_disabled(node) {
                self.flush(None);
            } else if let Some(text) = node.cast::<Text>() {
                if let Some(piece) = self.push(node, text.get()) {
                    piece.leading_wrapper = leading_wrapper;
                }
            } else if let Some(strong) = node.cast::<Strong>() {
                self.markup(strong.body(), leading_wrapper.or(Some(node)));
            } else if let Some(emph) = node.cast::<Emph>() {
                self.markup(emph.body(), leading_wrapper.or(Some(node)));
            } else if node.kind() == SyntaxKind::Space {
                // Soft source newlines supply context without becoming Unicode
                // paragraph separators. Layout still owns their preservation.
                self.push(node, " ");
                if self.pending_linebreak_break {
                    // The sentence break deferred across an attached `\` lands here.
                    self.pending_linebreak_break = false;
                    self.plan.breaks_mut(node).space = true;
                }
            } else if node.kind() == SyntaxKind::SmartQuote {
                self.push(node, node.leaf_text());
            } else if let Some(escape) = node.cast::<Escape>() {
                self.push(node, escape.get().encode_utf8(&mut [0; 4]));
            } else if let Some(shorthand) = node.cast::<Shorthand>() {
                self.push(node, shorthand.get().encode_utf8(&mut [0; 4]));
            } else if let Some(raw) = node.cast::<Raw>()
                && !raw.block()
            {
                for (index, line) in raw.lines().enumerate() {
                    if index > 0 {
                        self.push(node, "\n");
                    }
                    self.push(node, line.get());
                }
            } else if let Some(equation) = node.cast::<Equation>()
                && !equation.block()
            {
                self.flush(Some(node));
                // A literal punctuation atom is evidence at the math edge;
                // field-access dots and arbitrary math source are not prose.
                if let Some(last) = equation.body().to_untyped().children().last()
                    && last.kind() == SyntaxKind::MathText
                    && last
                        .leaf_text()
                        .ends_with(['.', '!', '?', '。', '！', '？'])
                {
                    self.push(node, last.leaf_text());
                }
            } else if matches!(node.kind(), SyntaxKind::Label | SyntaxKind::BlockComment) {
                // Metadata remains attached in source. A later whitespace node
                // provides the editable break, after these attachments.
            } else if node.kind() == SyntaxKind::Linebreak {
                if self.flush(Some(node)) {
                    // Keep `\` on the line of the sentence it terminates; the editable
                    // whitespace after it carries the break instead.
                    self.plan.breaks_mut(node).linebreak = true;
                    self.pending_linebreak_break = true;
                }
            } else if matches!(node.kind(), SyntaxKind::Parbreak | SyntaxKind::LineComment) {
                self.flush(None);
            } else {
                self.flush(Some(node));
            }
        }
    }

    /// Records the boundaries of the accumulated prose run and returns whether its
    /// last sentence ends with terminal punctuation.
    fn flush(&mut self, opaque: Option<&SyntaxNode>) -> bool {
        let ended_sentence = !has_protected_join(&self.text) && has_terminal_suffix(&self.text);
        for offset in self.segmenter.boundaries(&self.text) {
            // A no-break space is semantic content, even when ICU proposes a
            // sentence boundary after it. Do not replace it with source layout.
            let (before, after) = self.text.split_at(offset);
            if has_protected_join(before) || cannot_start_line(after) {
                continue;
            }
            let after_space = before.ends_with(|c: char| c.is_ascii_whitespace());
            if after_space
                || (!before.ends_with(char::is_whitespace)
                    && !after.starts_with(PROTECTED_JOINS)
                    && ends_with_cjk_terminator(before))
            {
                self.plan.record_boundary(&self.pieces, offset, after_space);
            }
        }
        if opaque.is_some()
            && ended_sentence
            && self.text.ends_with(|c: char| c.is_ascii_whitespace())
        {
            self.plan
                .record_boundary(&self.pieces, self.text.len(), true);
        }
        self.text.clear();
        self.pieces.clear();
        // A run ending at a structural boundary cannot carry an attached `\` into the
        // next run; the linebreak branch re-arms this after calling `flush`.
        self.pending_linebreak_break = false;
        ended_sentence
    }
}

fn has_protected_join(text: &str) -> bool {
    text.chars()
        .rev()
        .take_while(|c| c.is_whitespace() || *c == '\u{2060}')
        .any(|c| PROTECTED_JOINS.contains(&c))
}

fn cannot_start_line(text: &str) -> bool {
    text.split_ascii_whitespace()
        .next()
        .is_some_and(is_line_sensitive_markup_marker)
}
