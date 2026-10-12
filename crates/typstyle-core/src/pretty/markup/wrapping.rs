//! Markup layout for reflow and sentence wrapping. Sentence decisions come from
//! the contextual plan in the sibling segmentation adapter.

use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{MarkupLine, MarkupRepr, MarkupScope, segmentation::MarkupSegmentation};
use crate::{
    WrapMode,
    kind::SyntaxKindExt,
    pretty::{
        Context, PrettyPrinter,
        prelude::*,
        text::{is_line_sensitive_markup_marker, wrap_text},
    },
};

impl<'a> PrettyPrinter<'a> {
    /// Lays out a markup body in a reflowing mode.
    ///
    /// Sentence boundaries come from the contextual segmentation plan, and `fill` adds
    /// width-dependent reflow inside a sentence. Breaks use source whitespace or an unspaced
    /// CJK sentence boundary. Closing quotes, labels, and attached `\` stay with their sentence.
    pub(super) fn convert_markup_body_reflow(
        &'a self,
        ctx: Context,
        repr: &MarkupRepr<'a>,
        mode: WrapMode,
        segmentation: Option<&MarkupSegmentation>,
    ) -> ArenaDoc<'a> {
        let fill = matches!(mode, WrapMode::Fill | WrapMode::FillSentence);

        let mut doc = self.arena.nil();
        for (i, line) in repr.lines.iter().enumerate() {
            let &MarkupLine {
                ref nodes, breaks, ..
            } = line;
            for (j, node) in nodes.iter().enumerate() {
                if !ctx.break_suppressed && segmentation.is_some_and(|plan| plan.before(node)) {
                    doc += self.arena.hard_line();
                }
                if node.kind() == SyntaxKind::Space {
                    let next = nodes.get(j + 1);
                    let prev = j.checked_sub(1).and_then(|index| nodes.get(index));
                    doc += if next.is_some_and(|next| cannot_break_before_markup(next)) {
                        // Breaking here would turn the next token into a markup marker or
                        // separate a label from the text it annotates.
                        self.arena.space()
                    } else if !ctx.break_suppressed
                        && next.is_some_and(|next| {
                            next.kind() == SyntaxKind::Linebreak
                                && segmentation.is_some_and(|plan| plan.at_linebreak(next))
                        })
                    {
                        // The `\` attaches to the sentence before it, so this source space is
                        // replaced by the one the layout adds before `\`.
                        self.arena.nil()
                    } else if !ctx.break_suppressed
                        && segmentation.is_some_and(|plan| plan.at_space(node))
                        || fill
                            && (next.is_some_and(|next| reflow_prefers_exclusive(next))
                                || prev.is_some_and(|previous| reflow_prefers_exclusive(previous)))
                    {
                        // A sentence boundary starts a new line; block equations and raw blocks
                        // stay on a line of their own.
                        self.arena.hard_line()
                    } else if fill {
                        self.arena.softline()
                    } else {
                        // Sentence mode never reflows spaces within a line.
                        self.arena.space()
                    };
                    continue;
                }

                doc += if self.attr_store.is_format_disabled(node) {
                    self.convert_verbatim_untyped(node)
                } else if let Some(text) = node.cast::<Text>() {
                    // Split the leaf so each sentence can break independently.
                    convert_text(&self.arena, ctx, text, fill, segmentation)
                } else if !ctx.break_suppressed
                    && segmentation.is_some_and(|plan| plan.at_linebreak(node))
                {
                    // A forced linebreak that ends a sentence stays on that sentence's line;
                    // the following editable whitespace emits the sentence break. This is
                    // checked before the `Expr` branch because `Linebreak` casts to `Expr`.
                    self.arena.space() + self.convert_trivia_untyped(node)
                } else if let Some(expr) = node.cast::<Expr>() {
                    self.convert_wrapped_expr(ctx, expr, segmentation)
                } else if node.kind().is_comment() {
                    self.convert_comment(ctx, node)
                } else {
                    // Can be Hash, Semicolon, Shebang, Linebreak.
                    self.convert_trivia_untyped(node)
                };
            }
            let sentence_break = line
                .separator
                .is_some_and(|separator| segmentation.is_some_and(|plan| plan.at_space(separator)));
            // Should not eat trailing parbreaks.
            if breaks == 1
                && fill
                && !sentence_break
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
            }
        }
        doc
    }

    /// Converts a markup expression, sharing the sentence plan with transparent inline
    /// wrappers so a wrapper's end does not become an artificial sentence boundary.
    fn convert_wrapped_expr(
        &'a self,
        ctx: Context,
        expr: Expr<'a>,
        segmentation: Option<&MarkupSegmentation>,
    ) -> ArenaDoc<'a> {
        let Some(plan) = segmentation else {
            return self.convert_expr(ctx, expr);
        };
        let (markup, open, close) = match expr {
            Expr::Strong(strong) => (strong.body(), "*", "*"),
            Expr::Emph(emph) => (emph.body(), "_", "_"),
            _ => return self.convert_expr(ctx, expr),
        };
        self.convert_markup_with_segmentation(ctx, markup, MarkupScope::Strong, Some(plan))
            .enclose(open, close)
    }
}

/// Splits a text leaf at the sentence boundaries recorded in the plan. With `fill`, each
/// segment additionally soft-wraps between words.
fn convert_text<'a>(
    arena: &'a Arena<'a>,
    ctx: Context,
    text: Text<'a>,
    fill: bool,
    segmentation: Option<&MarkupSegmentation>,
) -> ArenaDoc<'a> {
    let mut doc = arena.nil();
    let source = text.get().as_str();
    let mut start = 0;
    let boundaries = segmentation.map_or(&[][..], |plan| plan.in_text(text.to_untyped()));
    for &end in boundaries {
        let segment = &source[start..end];
        let sentence = segment.trim_ascii_end();
        doc += if fill {
            wrap_text(arena, sentence)
        } else {
            arena.text(sentence)
        };
        if !ctx.break_suppressed {
            doc += arena.hard_line();
        } else if sentence.len() != segment.len() {
            doc += arena.space();
        }
        start = end;
    }
    let tail = &source[start..];
    doc + if fill {
        wrap_text(arena, tail)
    } else {
        arena.text(tail)
    }
}

/// For hard-line -> soft-line, keep a line exclusive (prevent soft breaks) when:
/// - It contains a block element that would lose its structure, or
/// - It contains only one non-text node, or
/// - It contains exactly two nodes where the first is a Hash, such as `#figure()`.
///
/// Comments and spaces written beside those nodes do not count.
fn reflow_preserve_exclusive(line: &MarkupLine) -> bool {
    let nodes = &line.nodes;
    match significant_nodes(nodes) {
        // A line holding only comments keeps its own line: joining it onto the next one would
        // turn that line into part of the comment.
        [] => !nodes.is_empty(),
        // Joining a block marker onto the previous line turns it into ordinary text, so its
        // structure is judged without the comments and spaces written beside it.
        [first, ..] if requires_own_line(first) => true,
        // A hash with its callee, such as `#figure()`.
        [first, _] if first.kind() == SyntaxKind::Hash => true,
        // A lone non-text node keeps its own line, but a comment beside it must not change how
        // the surrounding prose reflows.
        _ => nodes.len() == 1 && nodes[0].kind() != SyntaxKind::Text,
    }
}

/// Returns the nodes of a line without the spaces and comments around them.
///
/// `/* note */ = heading` is still a heading and `- item /* note */` is still a list item, so
/// trivia must not hide the structure that requires a line of its own.
fn significant_nodes<'a, 's>(nodes: &'s [&'a SyntaxNode]) -> &'s [&'a SyntaxNode] {
    let Some(start) = nodes
        .iter()
        .position(|node| !node.kind().is_space_or_comment())
    else {
        return &[];
    };
    let end = nodes
        .iter()
        .rposition(|node| !node.kind().is_space_or_comment())
        .unwrap_or(start);
    &nodes[start..=end]
}

/// Returns whether the node must start its own line: block markup, and the block equations and
/// raw blocks that lose their structure when they share a line.
fn requires_own_line(node: &SyntaxNode) -> bool {
    node.kind().is_block_markup() || reflow_prefers_exclusive(node)
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
fn is_block_equation(node: &SyntaxNode) -> bool {
    node.cast::<Equation>()
        .is_some_and(|equation| equation.block())
}

/// Returns whether the node is a raw block fenced with multiple backticks.
fn is_block_raw(node: &SyntaxNode) -> bool {
    node.cast::<Raw>().is_some_and(|raw| raw.block())
}

/// Do not introduce a line break before tokens that gain a special meaning at the start of a line.
/// Labels are kept on the same line as the preceding text as well.
fn cannot_break_before_markup(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::Label || is_line_sensitive_markup_marker(node.leaf_text())
}

#[cfg(test)]
mod tests {
    use typst_syntax::{Source, SyntaxKind, SyntaxNode};

    use crate::{Config, Typstyle, WrapMode};

    fn format_markup(input: &str, config: Config) -> String {
        // These wrapping assertions expect a newline-terminated source document.
        let input = if input.ends_with('\n') {
            input.to_string()
        } else {
            format!("{input}\n")
        };
        Typstyle::new(config)
            .format_source(Source::detached(input))
            .render()
            .unwrap()
    }

    /// Formats a whole document in fill mode at width 80, keeping its trailing newline.
    fn format_fill(input: &str) -> String {
        format_markup(
            input,
            Config::new().with_width(80).with_wrap_mode(WrapMode::Fill),
        )
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

    /// The kinds of the block elements of `source`, including nested ones.
    fn block_kinds(source: &str) -> Vec<SyntaxKind> {
        fn collect(node: &SyntaxNode, kinds: &mut Vec<SyntaxKind>) {
            if matches!(
                node.kind(),
                SyntaxKind::Heading
                    | SyntaxKind::ListItem
                    | SyntaxKind::EnumItem
                    | SyntaxKind::TermItem
            ) {
                kinds.push(node.kind());
            }
            for child in node.children() {
                collect(child, kinds);
            }
        }
        let source = Source::detached(source);
        let mut kinds = Vec::new();
        collect(source.root(), &mut kinds);
        kinds
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
    fn sentence_mode_requires_source_whitespace_before_non_text_nodes() {
        assert_eq!(format_sentences("A.#foo Next."), "A.#foo Next.\n");
        assert_eq!(format_sentences("A. #foo Next."), "A.\n#foo Next.\n");
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
    fn sentence_mode_keeps_pending_break_out_of_sibling_blocks() {
        // A trailing `\` and a following content block end the prose run; the deferred
        // break must not suppress the linebreak before the block.
        assert_eq!(
            format_sentences("Sentence. \\\n#block[content]"),
            "Sentence. \\\n#block[content]\n"
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

    #[test]
    fn fill_mode_keeps_nested_list_structure_next_to_comments() {
        // The comment trails a nested item, which must not join the item above it.
        let input = "- Testing\n  - Hello /* comment */\n  - World\n";
        assert_eq!(format_fill(input), input);
        assert_eq!(block_kinds(input).len(), 3);
    }

    #[test]
    fn fill_mode_keeps_comment_prefixed_calls_on_their_own_line() {
        // A call keeps its own line, and a leading comment must not merge it into the prose.
        let input = "Testing\n/* note */ #figure[Example]\n";
        assert_eq!(format_fill(input), input);
    }

    #[test]
    fn fill_mode_still_joins_comment_prefixed_prose() {
        // Comments around prose are not structural, so the surrounding text keeps reflowing.
        assert_eq!(
            format_fill("Testing\n/* note */ ordinary prose\n"),
            "Testing /* note */ ordinary prose\n"
        );
        assert_eq!(
            format_fill("Testing\n// note\nafter\n"),
            "Testing\n// note\nafter\n"
        );
    }

    /// The markers whose leading `-`, `+`, `/`, `=` or ordinal starts a block element.
    const BLOCK_MARKERS: [&str; 5] = [
        "= heading",
        "- item",
        "+ item",
        "1. item",
        "/ term: description",
    ];

    #[test]
    fn fill_mode_keeps_block_markers_on_their_own_line() {
        for marker in BLOCK_MARKERS {
            for comments in [
                // A leading comment must not let the marker join the preceding prose.
                "/* note */ ",
                "/* first */ /* second */ ",
                "// note\n",
            ] {
                for trailing in ["", " /* note */", " // note"] {
                    let input = format!("Testing\n{comments}{marker}{trailing}\n");
                    assert_eq!(format_fill(&input), input, "{marker:?}");
                }
            }
        }
    }

    #[test]
    fn fill_mode_preserves_block_structure_across_widths() {
        for mode in [WrapMode::Fill, WrapMode::FillSentence] {
            for width in [0, 20, 80] {
                let formatter = Typstyle::new(Config::new().with_width(width).with_wrap_mode(mode));
                for marker in BLOCK_MARKERS {
                    let input = format!("Testing\n/* note */ {marker}\n");
                    let output = formatter.format_text(&input).render().unwrap();
                    assert_eq!(
                        block_kinds(&output),
                        block_kinds(&input),
                        "{mode:?}, width={width}, {marker:?}: {output:?}"
                    );
                    assert_eq!(formatter.format_text(&output).render().unwrap(), output);
                }
            }
        }
    }
}
