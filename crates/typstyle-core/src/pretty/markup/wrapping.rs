use icu_segmenter::{SentenceSegmenter, SentenceSegmenterBorrowed};
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{MarkupLine, MarkupRepr};
use crate::pretty::{
    Context, PrettyPrinter,
    prelude::*,
    text::{is_enum_marker, wrap_text},
    util::is_comment_node,
};

impl<'a> PrettyPrinter<'a> {
    /// With text-wrapping enabled, spaces may turn to linebreaks, and linebreaks may turn to spaces, if safe.
    ///
    /// When `sentence_breaks` is set, sentence boundaries additionally become hardlines, so each
    /// sentence starts on its own line while long sentences still wrap to the line width.
    pub(super) fn convert_markup_body_wrapped(
        &'a self,
        ctx: Context,
        repr: &MarkupRepr<'a>,
        sentence_breaks: bool,
    ) -> ArenaDoc<'a> {
        let segmenter = sentence_breaks.then(|| SentenceSegmenter::new(Default::default()));

        let mut doc = self.arena.nil();
        let mut pending_sentence_break = false;
        for (i, line) in repr.lines.iter().enumerate() {
            let &MarkupLine {
                ref nodes, breaks, ..
            } = line;
            for (j, node) in nodes.iter().enumerate() {
                if node.kind() == SyntaxKind::Space {
                    doc += if nodes
                        .get(j + 1)
                        .is_some_and(|node| cannot_break_before_markup(node))
                    {
                        pending_sentence_break = false;
                        self.arena.space()
                    } else if pending_sentence_break {
                        pending_sentence_break = false;
                        self.arena.hard_line()
                    } else if nodes
                        .get(j + 1)
                        .is_some_and(|node| reflow_prefers_exclusive(node))
                        || nodes
                            .get(j - 1)
                            .is_some_and(|node| reflow_prefers_exclusive(node))
                    {
                        self.arena.hard_line()
                    } else {
                        self.arena.softline()
                    };
                    continue;
                }

                // A closing quote stays glued to the sentence it terminates, so the pending break
                // survives until the following whitespace.
                let defer_break = sentence_breaks && is_sentence_closer(node);
                let leading_break = if pending_sentence_break && !defer_break {
                    self.arena.hard_line()
                } else {
                    self.arena.nil()
                };

                let node_doc = if let Some(text) = node.cast::<Text>() {
                    if let Some(segmenter) = &segmenter {
                        let (text_doc, ended_sentence) =
                            convert_text_sentence_split(&self.arena, segmenter, text, true);
                        pending_sentence_break = ended_sentence;
                        text_doc
                    } else {
                        self.convert_text_wrapped(text)
                    }
                } else if let Some(expr) = node.cast::<Expr>() {
                    if !defer_break {
                        pending_sentence_break =
                            sentence_breaks && inline_node_ends_with_sentence(node);
                    }
                    self.convert_expr(ctx, expr)
                } else if is_comment_node(node) {
                    pending_sentence_break = false;
                    self.convert_comment(ctx, node)
                } else {
                    // can be Hash, Semicolon, Shebang
                    pending_sentence_break =
                        sentence_breaks && inline_node_ends_with_sentence(node);
                    self.convert_trivia_untyped(node)
                };

                doc += leading_break + node_doc;
            }
            // Should not eat trailing parbreaks.
            if breaks == 1
                && !pending_sentence_break
                && i + 1 != repr.lines.len()
                && !nodes.last().is_some_and(|last| {
                    reflow_should_break_after(last) || reflow_preserve_break_after(last)
                })
                && !reflow_preserve_exclusive(line)
                && !reflow_preserve_exclusive(&repr.lines[i + 1])
            {
                doc += self.arena.softline();
            } else if breaks > 0 {
                doc += self.arena.hard_line().repeat(breaks);
                pending_sentence_break = false;
            }
        }
        doc
    }

    /// With sentence-per-line mode, split sentence boundaries inside text leaves.
    pub(super) fn convert_markup_body_sentence_per_line(
        &'a self,
        ctx: Context,
        repr: &MarkupRepr<'a>,
    ) -> ArenaDoc<'a> {
        let segmenter = SentenceSegmenter::new(Default::default());

        let mut doc = self.arena.nil();
        let mut pending_sentence_break = false;
        for line in repr.lines.iter() {
            for (index, node) in line.nodes.iter().enumerate() {
                doc += if node.kind() == SyntaxKind::Space {
                    if pending_sentence_break {
                        pending_sentence_break = false;
                        if line
                            .nodes
                            .get(index + 1)
                            .is_some_and(|node| cannot_break_before_markup(node))
                        {
                            self.arena.space()
                        } else {
                            self.arena.hard_line()
                        }
                    } else {
                        self.arena.space()
                    }
                } else if let Some(text) = node.cast::<Text>() {
                    let (text_doc, ended_sentence) =
                        convert_text_sentence_split(&self.arena, &segmenter, text, false);
                    let leading_break = if pending_sentence_break {
                        self.arena.hard_line()
                    } else {
                        self.arena.nil()
                    };
                    pending_sentence_break = ended_sentence;
                    leading_break + text_doc
                } else if is_sentence_closer(node) {
                    self.convert_trivia_untyped(node)
                } else if let Some(expr) = node.cast::<Expr>() {
                    let leading_break = if pending_sentence_break {
                        self.arena.hard_line()
                    } else {
                        self.arena.nil()
                    };
                    pending_sentence_break = inline_node_ends_with_sentence(node);
                    leading_break + self.convert_expr(ctx, expr)
                } else if is_comment_node(node) {
                    pending_sentence_break = false;
                    self.convert_comment(ctx, node)
                } else {
                    let leading_break = if pending_sentence_break {
                        self.arena.hard_line()
                    } else {
                        self.arena.nil()
                    };
                    pending_sentence_break = inline_node_ends_with_sentence(node);
                    leading_break + self.convert_trivia_untyped(node)
                };
            }
            if line.breaks > 0 {
                doc += self.arena.hard_line().repeat(line.breaks);
                pending_sentence_break = false;
            }
        }

        doc
    }
}

// Text conversion helper.

/// Splits `text` at sentence boundaries. With `fill`, each sentence additionally soft-wraps
/// between words. Returns the doc and whether the text ends a sentence.
fn convert_text_sentence_split<'a>(
    arena: &'a Arena<'a>,
    segmenter: &SentenceSegmenterBorrowed,
    text: Text<'a>,
    fill: bool,
) -> (ArenaDoc<'a>, bool) {
    let text = text.get();
    let mut boundaries = segmenter.segment_str(text);
    let Some(mut start) = boundaries.next() else {
        return (arena.nil(), false);
    };
    let mut doc = arena.nil();
    let mut first = true;
    let mut ended_sentence = false;
    let mut previous_was_abbreviation = false;

    for end in boundaries {
        let sentence = text[start..end].trim();
        if !sentence.is_empty() {
            if !first {
                doc += if cannot_break_before_text(sentence) {
                    arena.space()
                } else if previous_was_abbreviation {
                    if fill {
                        arena.softline()
                    } else {
                        arena.space()
                    }
                } else {
                    arena.hard_line()
                };
            }
            doc += if fill {
                wrap_text(arena, sentence)
            } else {
                arena.text(sentence)
            };
            if end == text.len() && text.ends_with(' ') {
                doc += if fill {
                    arena.softline()
                } else {
                    arena.space()
                };
            }
            first = false;
            previous_was_abbreviation = ends_with_common_abbreviation(sentence);
            ended_sentence = source_ends_with_sentence(sentence);
        }
        start = end;
    }

    (doc, ended_sentence)
}

/// For hard-line -> soft-line, keep a line exclusive (prevent soft breaks) when:
/// - It contains only one non-text node, or
/// - It contains exactly two nodes where the first is a Hash, such as `#figure()`.
fn reflow_preserve_exclusive(line: &MarkupLine) -> bool {
    let nodes = &line.nodes;
    let len = nodes.len();
    len == 1 && nodes[0].kind() != SyntaxKind::Text
        || len == 2 && nodes[0].kind() == SyntaxKind::Hash
        || len > 0 && reflow_prefers_exclusive(nodes[0])
}

/// For space -> hard-line, prefer block equations and raw blocks exclusive to a single line.
fn reflow_prefers_exclusive(node: &SyntaxNode) -> bool {
    is_block_equation(node) || is_block_raw(node)
}

/// For hard-line -> soft-line, always break after block elements or line comments.
fn reflow_should_break_after(node: &SyntaxNode) -> bool {
    matches!(
        node.kind(),
        SyntaxKind::Heading
            | SyntaxKind::ListItem
            | SyntaxKind::EnumItem
            | SyntaxKind::TermItem
            | SyntaxKind::LineComment
    )
}

/// For hard-line -> soft-line, preserve breaks after nodes where breaking is visually better.
fn reflow_preserve_break_after(node: &SyntaxNode) -> bool {
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

fn is_block_equation(it: &SyntaxNode) -> bool {
    it.cast::<Equation>()
        .is_some_and(|equation| equation.block())
}

fn is_block_raw(it: &SyntaxNode) -> bool {
    it.cast::<Raw>().is_some_and(|raw| raw.block())
}

/// A closing smart quote belongs to the preceding sentence. Keep a pending break until its
/// following whitespace so the quote remains on the same line as its sentence.
fn is_sentence_closer(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::SmartQuote
}

/// Do not introduce a line break before tokens that gain a special meaning at the start of a line.
/// Labels are kept on the same line as the preceding text as well.
fn cannot_break_before_markup(node: &SyntaxNode) -> bool {
    let text = node.leaf_text();
    is_line_sensitive_markup_marker(text)
        || matches!(node.kind(), SyntaxKind::Label)
        || is_enum_marker(text)
}

/// Only inline markup wrappers contribute their terminal punctuation to the surrounding
/// sentence. Arbitrary expressions and content blocks can end with punctuation while still being
/// followed by prose in the same sentence.
fn inline_node_ends_with_sentence(node: &SyntaxNode) -> bool {
    (node.is::<Strong>() || node.is::<Emph>() || node.is::<Raw>() || node.is::<Equation>())
        && source_ends_with_sentence(&node.full_text())
}

/// Do not introduce a line break before text that becomes a markup marker at the start of a line.
fn cannot_break_before_text(text: &str) -> bool {
    let Some(first) = text.split_ascii_whitespace().next() else {
        return false;
    };
    is_line_sensitive_markup_marker(first) || is_enum_marker(first)
}

fn is_line_sensitive_markup_marker(text: &str) -> bool {
    matches!(text, "+" | "-" | "/") || !text.is_empty() && text.chars().all(|c| c == '=')
}

fn source_ends_with_sentence(text: &str) -> bool {
    sentence_ends_with_punctuation(text) && !ends_with_common_abbreviation(text)
}

fn sentence_ends_with_punctuation(text: &str) -> bool {
    trim_sentence_closers(text).ends_with(['.', '!', '?', '。', '！', '？'])
}

fn is_boundary_punctuation(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '(' | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '（'
                | '）'
                | '【'
                | '】'
                | '『'
                | '』'
                | '「'
                | '」'
                | '〉'
                | '》'
                | '"'
                | '\''
                | '”'
                | '“'
                | '’'
                | '‘'
                | '»'
                | '«'
                | '›'
                | '‹'
                | '*'
                | '_'
                | '`'
                | '$'
                | ','
                | ';'
                | ':'
        )
}

fn trim_sentence_closers(text: &str) -> &str {
    text.trim_end_matches(is_boundary_punctuation)
}

fn ends_with_common_abbreviation(text: &str) -> bool {
    trim_sentence_closers(text)
        .split_ascii_whitespace()
        .next_back()
        .is_some_and(is_common_abbreviation)
}

fn is_common_abbreviation(text: &str) -> bool {
    let clean = text.trim_matches(is_boundary_punctuation);
    let lower = clean.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "dr."
            | "mr."
            | "mrs."
            | "ms."
            | "prof."
            | "sr."
            | "jr."
            | "st."
            | "vs."
            | "etc."
            | "cf."
            | "viz."
            | "ibid."
            | "ca."
            | "eq."
            | "ref."
            | "e.g."
            | "i.e."
            | "al."
            | "vol."
            | "ed."
            | "pp."
            | "fig."
            | "sec."
            | "approx."
    ) || is_multidot_acronym(clean)
}

fn is_multidot_acronym(s: &str) -> bool {
    if !s.ends_with('.') {
        return false;
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() >= 3 {
        let non_empty = &parts[..parts.len() - 1];
        non_empty.iter().all(|p| {
            !p.is_empty() && p.chars().count() <= 2 && p.chars().all(|c| c.is_alphanumeric())
        })
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use typst_syntax::Source;

    use crate::{Config, Typstyle, WrapMode};

    fn format_markup(input: &str, config: Config) -> String {
        // These wrapping assertions expect a newline-terminated source document.
        Typstyle::new(config)
            .format_source(Source::detached(format!("{input}\n")))
            .render()
            .unwrap()
    }

    fn format_sentences(input: &str) -> String {
        format_markup(input, Config::new().with_wrap_mode(WrapMode::Sentence))
    }

    fn format_fill_sentences(input: &str, width: usize) -> String {
        format_markup(
            input,
            Config::new()
                .with_width(width)
                .with_wrap_mode(WrapMode::FillSentence),
        )
    }

    #[test]
    fn fill_mode_preserves_multilevel_heading_marker() {
        assert_eq!(
            format_markup(
                "A. == ordinary prose.",
                Config::new().with_width(3).with_wrap_mode(WrapMode::Fill),
            ),
            "A. ==\nordinary\nprose.\n"
        );
    }

    #[test]
    fn sentence_mode_preserves_line_sensitive_markup() {
        for input in [
            "A. - this is ordinary prose.",
            "A. = this is ordinary prose.",
            "A. == this is ordinary prose.",
            "A. + this is ordinary prose.",
            "A. / term: this is ordinary prose.",
            "A. <label> this is ordinary prose.",
            "A. 1. this is ordinary prose.",
        ] {
            assert_eq!(format_sentences(input), format!("{input}\n"));
        }
    }

    #[test]
    fn sentence_mode_detects_inline_sentence_endings() {
        assert_eq!(
            format_sentences("Hello *world.* Next."),
            "Hello *world.*\nNext.\n"
        );
        assert_eq!(
            format_sentences("Hello `world.` Next."),
            "Hello `world.`\nNext.\n"
        );
        assert_eq!(format_sentences("Hello $x.$ Next."), "Hello $x.$\nNext.\n");
    }

    #[test]
    fn sentence_mode_keeps_abbreviations_with_inline_continuations() {
        assert_eq!(
            format_sentences("Smith et al. *argue* this. Next."),
            "Smith et al. *argue* this.\nNext.\n"
        );
        assert_eq!(
            format_sentences("See fig. #ref here. Next."),
            "See fig. #ref here.\nNext.\n"
        );
        assert_eq!(
            format_sentences("Smith *et al.* argue this. Next."),
            "Smith *et al.* argue this.\nNext.\n"
        );
    }

    #[test]
    fn sentence_mode_handles_multidot_abbreviations() {
        assert_eq!(
            format_sentences("We use tools, e.g., typstyle. Next."),
            "We use tools, e.g., typstyle.\nNext.\n"
        );
        assert_eq!(
            format_sentences("For example (i.e., this feature). Next."),
            "For example (i.e., this feature).\nNext.\n"
        );
        assert_eq!(
            format_sentences("Dies ist z.B. ein Test. Nächster Satz."),
            "Dies ist z.B. ein Test.\nNächster Satz.\n"
        );
    }

    #[test]
    fn sentence_mode_keeps_closing_quotes_with_the_sentence() {
        assert_eq!(
            format_sentences("\"Hello.\" World."),
            "\"Hello.\"\nWorld.\n"
        );
    }

    #[test]
    fn sentence_mode_carries_breaks_across_non_text_nodes() {
        assert_eq!(format_sentences("A.#foo Next."), "A.\n#foo Next.\n");
    }

    #[test]
    fn fill_sentence_mode_splits_sentences_that_would_fit() {
        assert_eq!(
            format_fill_sentences("A short one. Another.", 80),
            "A short one.\nAnother.\n"
        );
    }

    #[test]
    fn fill_sentence_mode_joins_mid_sentence_line_breaks() {
        assert_eq!(
            format_fill_sentences(
                "A sentence broken\nacross lines. Next one\nalso broken.",
                80
            ),
            "A sentence broken across lines.\nNext one also broken.\n"
        );
    }

    #[test]
    fn fill_sentence_mode_wraps_long_sentences_to_width() {
        assert_eq!(
            format_fill_sentences("aaa bbb ccc ddd eee fff. Next.", 15),
            "aaa bbb ccc ddd\neee fff.\nNext.\n"
        );
    }

    #[test]
    fn fill_sentence_mode_preserves_paragraph_breaks() {
        assert_eq!(
            format_fill_sentences("One. Two.\n\nThree.", 80),
            "One.\nTwo.\n\nThree.\n"
        );
    }

    #[test]
    fn fill_sentence_mode_preserves_line_sensitive_markup() {
        for input in [
            "A. - this is ordinary prose.",
            "A. == this is ordinary prose.",
            "A. 1. this is ordinary prose.",
        ] {
            assert_eq!(format_fill_sentences(input, 80), format!("{input}\n"));
        }
    }

    #[test]
    fn fill_sentence_mode_keeps_abbreviations_and_closing_quotes() {
        assert_eq!(
            format_fill_sentences("Smith et al. argue this. Next.", 80),
            "Smith et al. argue this.\nNext.\n"
        );
        assert_eq!(
            format_fill_sentences("\"Hello.\" World.", 80),
            "\"Hello.\"\nWorld.\n"
        );
    }
}
