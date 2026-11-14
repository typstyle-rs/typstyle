//! Implements the markup body layout for the text-wrapping modes: re-flowing prose
//! across lines and laying out one sentence per line.

use icu_segmenter::{SentenceSegmenter, SentenceSegmenterBorrowed};
use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{MarkupLine, MarkupRepr};
use crate::{
    WrapMode,
    kind::SyntaxKindExt,
    pretty::{
        Context, PrettyPrinter,
        prelude::*,
        text::{is_enum_marker, wrap_text},
    },
};

impl<'a> PrettyPrinter<'a> {
    /// Lays out a markup body in a reflowing mode.
    ///
    /// Spaces may turn to linebreaks and source linebreaks may turn to spaces, when safe. With
    /// sentence breaks, each sentence starts on its own line; when filling, long sentences still
    /// wrap to the line width.
    ///
    /// A sentence boundary detected inside a node (for example at the end of a text leaf) is
    /// carried in `pending_sentence_break` until the next break point, where it becomes a hard
    /// line. This lets a trailing quote or an attached `\` stay with the sentence first.
    pub(super) fn convert_markup_body_reflow(
        &'a self,
        ctx: Context,
        repr: &MarkupRepr<'a>,
        mode: WrapMode,
    ) -> ArenaDoc<'a> {
        let fill = matches!(mode, WrapMode::Fill | WrapMode::FillSentence);
        let sentence_breaks = matches!(mode, WrapMode::Sentence | WrapMode::FillSentence);
        let segmenter = sentence_breaks.then(|| SentenceSegmenter::new(Default::default()));

        let mut doc = self.arena.nil();
        // Carries a sentence boundary to the next break point instead of breaking immediately.
        let mut pending_sentence_break = false;
        for (i, line) in repr.lines.iter().enumerate() {
            let &MarkupLine {
                ref nodes, breaks, ..
            } = line;
            for (j, node) in nodes.iter().enumerate() {
                if node.kind() == SyntaxKind::Space {
                    let next = nodes.get(j + 1);
                    let prev = j.checked_sub(1).and_then(|k| nodes.get(k));
                    doc += if next.is_some_and(|node| cannot_break_before_markup(node)) {
                        // Breaking here would turn the next token into a markup marker or
                        // separate a label from the text it annotates.
                        pending_sentence_break = false;
                        self.arena.space()
                    } else if pending_sentence_break
                        && prev.is_none_or(|node| node.kind() != SyntaxKind::Linebreak)
                        && next.is_some_and(|node| node.kind() == SyntaxKind::Linebreak)
                    {
                        // The `\` that follows attaches to the previous sentence, so let it
                        // supply the separator and keep the pending break unresolved.
                        self.arena.nil()
                    } else if pending_sentence_break {
                        // The previous sentence ended: start the next one on a new line.
                        pending_sentence_break = false;
                        self.arena.hard_line()
                    } else if !fill {
                        // Sentence mode never reflows spaces within a line.
                        self.arena.space()
                    } else if next.is_some_and(|node| reflow_prefers_exclusive(node))
                        || prev.is_some_and(|node| reflow_prefers_exclusive(node))
                    {
                        // Keep block equations and raw blocks on a line of their own.
                        self.arena.hard_line()
                    } else {
                        self.arena.softline()
                    };
                    continue;
                }

                // A closing quote stays glued to the sentence it terminates, so the pending break
                // survives until the following whitespace.
                let defer_break = sentence_breaks && is_sentence_closer(node);
                // A linebreak that follows a sentence stays on that sentence's line. The break
                // itself is emitted by the following space or source line break.
                let attach_linebreak = sentence_breaks
                    && node.kind() == SyntaxKind::Linebreak
                    && pending_sentence_break;
                // Both cases postpone resolving the pending break until after this node.
                let keep_pending = attach_linebreak || defer_break;
                let leading_break = if attach_linebreak {
                    // Glue the `\` to the sentence with a space instead of breaking first.
                    self.arena.space()
                } else if pending_sentence_break && !defer_break {
                    self.arena.hard_line()
                } else {
                    self.arena.nil()
                };

                let node_doc = if let Some(text) = node.cast::<Text>() {
                    if let Some(segmenter) = &segmenter {
                        // Split the leaf so each sentence can break independently.
                        let (text_doc, ended_sentence) =
                            convert_text_sentence_split(&self.arena, segmenter, text, fill);
                        pending_sentence_break = ended_sentence;
                        text_doc
                    } else {
                        self.convert_text_wrapped(text)
                    }
                } else if let Some(expr) = node.cast::<Expr>() {
                    // Only inline wrappers contribute their terminal punctuation to the
                    // sentence; other expressions are treated as prose content.
                    if !keep_pending {
                        pending_sentence_break =
                            sentence_breaks && inline_node_ends_with_sentence(node);
                    }
                    self.convert_expr(ctx, expr)
                } else if node.kind().is_comment() {
                    // A comment interrupts the surrounding sentence.
                    pending_sentence_break = false;
                    self.convert_comment(ctx, node)
                } else {
                    // Can be Hash, Semicolon, Shebang.
                    pending_sentence_break =
                        sentence_breaks && inline_node_ends_with_sentence(node);
                    self.convert_trivia_untyped(node)
                };

                doc += leading_break + node_doc;
            }
            // Should not eat trailing parbreaks.
            if breaks == 1
                && fill
                && !pending_sentence_break
                && i + 1 != repr.lines.len()
                && !nodes
                    .last()
                    .is_some_and(|last| reflow_keeps_break_after(last))
                && !reflow_preserve_exclusive(line)
                && !reflow_preserve_exclusive(&repr.lines[i + 1])
            {
                // A lone source linebreak may reflow into a space, unless a sentence
                // boundary, a structural node, or an exclusive line requires a hardline.
                doc += self.arena.softline();
            } else if breaks > 0 {
                // Keep paragraph breaks and any mandatory linebreaks.
                doc += self.arena.hard_line().repeat(breaks);
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
    // Whether the previous segment ended with a common abbreviation (e.g. `etc.`), in which
    // case its dot does not end the sentence.
    let mut previous_was_abbreviation = false;

    for end in boundaries {
        let sentence = text[start..end].trim();
        if !sentence.is_empty() {
            if !first {
                doc += if cannot_break_before_text(sentence) {
                    // Splitting here would turn the sentence into markup, so keep it inline.
                    arena.space()
                } else if previous_was_abbreviation {
                    // The previous segment ended in an abbreviation, not a sentence boundary.
                    soft_or_space(arena, fill)
                } else {
                    arena.hard_line()
                };
            }
            doc += if fill {
                wrap_text(arena, sentence)
            } else {
                arena.text(sentence)
            };
            // Preserve a trailing space at the end of the leaf.
            if end == text.len() && text.ends_with(' ') {
                doc += soft_or_space(arena, fill);
            }
            first = false;
            previous_was_abbreviation = is_common_abbreviation(sentence);
            ended_sentence = source_ends_with_sentence(sentence);
        }
        start = end;
    }

    (doc, ended_sentence)
}

/// A soft line when filling text, a plain space otherwise.
fn soft_or_space<'a>(arena: &'a Arena<'a>, fill: bool) -> ArenaDoc<'a> {
    if fill {
        arena.softline()
    } else {
        arena.space()
    }
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

/// For hard-line -> soft-line, keep a break after nodes where breaking is visually better:
/// block elements, line comments, and structural nodes.
fn reflow_keeps_break_after(node: &SyntaxNode) -> bool {
    let kind = node.kind();
    kind.is_block_markup()
        || kind.is_comment()
        || matches!(
            kind,
            SyntaxKind::Linebreak
                | SyntaxKind::Label
                | SyntaxKind::CodeBlock
                | SyntaxKind::ContentBlock
                | SyntaxKind::Conditional
                | SyntaxKind::WhileLoop
                | SyntaxKind::ForLoop
                | SyntaxKind::Contextual
        )
        || is_block_equation(node)
        || is_block_raw(node)
}

/// Returns whether the node is an equation in block (display) style.
fn is_block_equation(it: &SyntaxNode) -> bool {
    it.cast::<Equation>()
        .is_some_and(|equation| equation.block())
}

/// Returns whether the node is a raw block fenced with multiple backticks.
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

/// Returns whether the text would start a list item, term, or heading if it began a
/// line: `+`, `-`, `/`, or a run of `=`.
fn is_line_sensitive_markup_marker(text: &str) -> bool {
    matches!(text, "+" | "-" | "/") || !text.is_empty() && text.chars().all(|c| c == '=')
}

/// Returns whether the source text ends a sentence: it ends with sentence punctuation
/// that is not part of a common abbreviation.
fn source_ends_with_sentence(text: &str) -> bool {
    sentence_ends_with_punctuation(text) && !ends_with_common_abbreviation(text)
}

/// Returns whether the text ends with sentence punctuation, ignoring trailing
/// whitespace and closing brackets, quotes, and markup delimiters.
fn sentence_ends_with_punctuation(text: &str) -> bool {
    trim_sentence_closers(text).ends_with(['.', '!', '?', '。', '！', '？'])
}

/// Strips trailing whitespace and closing brackets, quotes, and markup delimiters that
/// may sit between the last word and the sentence-ending punctuation.
fn trim_sentence_closers(text: &str) -> &str {
    text.trim_end_matches(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                ')' | ']'
                    | '}'
                    | '）'
                    | '】'
                    | '』'
                    | '」'
                    | '〉'
                    | '》'
                    | '"'
                    | '\''
                    | '”'
                    | '’'
                    | '»'
                    | '›'
                    | '*'
                    | '_'
                    | '`'
                    | '$'
            )
    })
}

/// Returns whether the last word of the text is a common abbreviation (e.g., `etc.`),
/// so its trailing dot does not end a sentence.
fn ends_with_common_abbreviation(text: &str) -> bool {
    trim_sentence_closers(text)
        .split_ascii_whitespace()
        .next_back()
        .is_some_and(is_common_abbreviation)
}

/// Returns whether the word is a common abbreviation whose dot does not terminate a
/// sentence.
fn is_common_abbreviation(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
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
            | "e.g."
            | "i.e."
            | "al."
            | "vol."
            | "ed."
            | "pp."
            | "fig."
            | "sec."
            | "approx."
    )
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
    fn sentence_mode_keeps_linebreak_with_preceding_sentence() {
        assert_eq!(
            format_sentences("Sentence 1.\\ Sentence 2."),
            "Sentence 1. \\\nSentence 2.\n"
        );
        assert_eq!(
            format_sentences("Sentence 3. \\ Sentence 4."),
            "Sentence 3. \\\nSentence 4.\n"
        );
        assert_eq!(
            format_sentences("Sentence 5. \\\nSentence 6."),
            "Sentence 5. \\\nSentence 6.\n"
        );
        assert_eq!(
            format_sentences("\"Hello.\" \\ World."),
            "\"Hello.\" \\\nWorld.\n"
        );
        assert_eq!(format_sentences("A. \\ \\ B."), "A. \\\n\\ B.\n");
    }

    #[test]
    fn sentence_mode_leaves_leading_and_mid_sentence_linebreaks() {
        assert_eq!(format_sentences("\\ Sentence 2."), "\\ Sentence 2.\n");
        assert_eq!(
            format_sentences("Hello\\ World. Next."),
            "Hello\\ World.\nNext.\n"
        );
    }

    #[test]
    fn fill_sentence_mode_keeps_linebreak_with_preceding_sentence() {
        assert_eq!(
            format_fill_sentences("Sentence 1.\\ Sentence 2.", 80),
            "Sentence 1. \\\nSentence 2.\n"
        );
        assert_eq!(
            format_fill_sentences("Sentence 3. \\ Sentence 4.", 80),
            "Sentence 3. \\\nSentence 4.\n"
        );
        assert_eq!(
            format_fill_sentences("Sentence 5. \\\nSentence 6.", 80),
            "Sentence 5. \\\nSentence 6.\n"
        );
        assert_eq!(
            format_fill_sentences("\"Hello.\" \\ World.", 80),
            "\"Hello.\" \\\nWorld.\n"
        );
        assert_eq!(format_fill_sentences("A. \\ \\ B.", 80), "A. \\\n\\ B.\n");
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
