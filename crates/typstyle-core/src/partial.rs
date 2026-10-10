//! Range-based formatting and analysis utilities for Typstyle.
//!
//! All byte ranges in this module are specified as UTF-8 offsets.
//! If you have a UTF-16 range or line-column range (as used in LSP or some editors),
//! you must convert it to a UTF-8 byte offset before calling these functions.
//! The `typst_syntax::Source` API provides helpers for converting between line-column,
//! UTF-16, and UTF-8 offsets.

use std::ops::Range;

use typst_syntax::{LinkedNode, Source, SyntaxKind, SyntaxNode, ast::*};

use crate::{
    AttrStore, Error, PrettyPrinter, Typstyle,
    pretty::{MarkupScope, Mode},
    utils::{self, indent_4_to_2},
};

/// Result of a range-based formatting or analysis operation.
#[derive(Debug, Clone)]
pub struct RangeResult {
    /// The actual source range that was processed (may be larger than requested to include complete nodes).
    pub source_range: Range<usize>,
    /// The output for the range (formatted text, IR, AST, etc.).
    pub content: String,
}

impl RangeResult {
    fn empty(pos: usize) -> Self {
        Self {
            source_range: pos..pos,
            content: String::new(),
        }
    }
}

enum RangeOutput {
    Text,
    Ir,
}

impl Typstyle {
    /// Format the smallest syntax node that fully covers the given byte range.
    ///
    /// The formatted range may be larger than the input to ensure valid syntax.
    /// Whitespace handling matches [`crate::Formatter::render`], but line endings
    /// follow the selected node's layout; code fragments do not receive an extra
    /// final newline solely because they are formatted.
    ///
    /// # Arguments
    /// - `source`: The source code.
    /// - `utf8_range`: The UTF-8 byte range to format.
    ///
    /// # Returns
    /// A `RangeResult` with the formatted text and actual node range.
    pub fn format_source_range(
        &self,
        source: Source,
        utf8_range: Range<usize>,
    ) -> Result<RangeResult, Error> {
        self.format_source_range_impl(source, utf8_range, RangeOutput::Text)
    }

    /// Get the pretty IR for the smallest syntax node covering the given byte range.
    ///
    /// # Arguments
    /// - `source`: The source code.
    /// - `utf8_range`: The UTF-8 byte range to analyze.
    ///
    /// # Returns
    /// A `RangeResult` with the IR and actual node range.
    pub fn format_source_range_ir(
        &self,
        source: Source,
        utf8_range: Range<usize>,
    ) -> Result<RangeResult, Error> {
        self.format_source_range_impl(source, utf8_range, RangeOutput::Ir)
    }

    fn format_source_range_impl(
        &self,
        source: Source,
        utf8_range: Range<usize>,
        output: RangeOutput,
    ) -> Result<RangeResult, Error> {
        // An exact node selection retains that node's boundaries. Otherwise, whitespace
        // around the requested syntax remains outside the replacement.
        let mut node =
            get_node_cover_range(&source, utf8_range.clone()).ok_or(Error::SyntaxError)?;
        let selection_range = if node.range() == utf8_range {
            utf8_range.clone()
        } else {
            trim_range(source.text(), utf8_range.clone())
        };
        if !utf8_range.is_empty() && selection_range.is_empty() {
            return Ok(RangeResult::empty(selection_range.start));
        }
        if selection_range != utf8_range {
            node =
                get_node_cover_range(&source, selection_range.clone()).ok_or(Error::SyntaxError)?;
        }
        // Keep enclosing nodes with the same boundaries, including the document root.
        while let Some(parent) = node
            .parent()
            .filter(|parent| parent.range() == selection_range)
        {
            node = parent.clone();
        }
        if node.diagnosis().errors {
            return Err(Error::SyntaxError);
        }
        if node.kind() == SyntaxKind::Space {
            return Ok(RangeResult::empty(selection_range.start));
        }

        // Directives on siblings and ancestors also apply to a selected subtree.
        let mut attrs = AttrStore::for_partial_formatting_context(source.root());
        let mut ancestors = std::iter::successors(Some(&node), |node| node.parent());
        let mode = ancestors
            .clone()
            .find_map(|node| match node.kind() {
                SyntaxKind::Markup => Some(Mode::Markup),
                SyntaxKind::CodeBlock => Some(Mode::Code),
                SyntaxKind::Equation => Some(Mode::Math),
                _ => None,
            })
            .unwrap_or(Mode::Markup);
        let disabled = ancestors.any(|node| attrs.is_format_disabled(node.get()));

        let (node, node_range) = if disabled {
            (node.get().clone(), node.range())
        } else {
            let Some(refined) = refine_node_range(node, selection_range.clone()) else {
                return Ok(RangeResult::empty(selection_range.start)); // No edit
            };
            refined
        };
        let markup_scope = if node.span() == source.root().span() {
            MarkupScope::Document
        } else {
            MarkupScope::Fragment
        };
        if markup_scope != MarkupScope::Document
            && source.text()[node_range.clone()].trim().is_empty()
        {
            return Ok(RangeResult::empty(selection_range.start));
        }

        attrs.compute_for_subtree(&node);
        let printer = PrettyPrinter::new(self.config.clone(), attrs);
        let doc = if disabled {
            printer.convert_verbatim_text(&source.text()[node_range.clone()])
        } else {
            printer.try_convert_with_scope(&node, mode, markup_scope)?
        };

        // Infer indent from context.
        let indent = utils::count_spaces_after_last_newline(source.text(), node_range.start);
        let doc = doc.nest(indent as isize);
        let content = match output {
            RangeOutput::Text => doc.print_with(printer.render_options()).to_string(),
            RangeOutput::Ir => indent_4_to_2(&format!("{doc:#?}")),
        };

        Ok(RangeResult {
            source_range: node_range,
            content,
        })
    }
}

/// Formats the smallest syntax node covering the given byte range as a debug AST string
/// with 2-space indentation. Returns the node's actual source range and formatted AST.
///
/// # Arguments
/// - `source`: The source code.
/// - `utf8_range`: The UTF-8 byte range to analyze.
///
/// # Returns
/// A `RangeResult` with the node's range and formatted AST.
pub fn format_range_ast(source: &Source, utf8_range: Range<usize>) -> Result<RangeResult, Error> {
    let node = get_node_cover_range(source, trim_range(source.text(), utf8_range))
        .filter(|node| !node.diagnosis().errors)
        .ok_or(Error::SyntaxError)?;
    Ok(RangeResult {
        source_range: node.range(),
        content: indent_4_to_2(&format!("{node:#?}")),
    })
}

fn refine_node_range(node: LinkedNode, range: Range<usize>) -> Option<(SyntaxNode, Range<usize>)> {
    if node.range() == range {
        return Some((node.get().clone(), range));
    }
    match node.kind() {
        SyntaxKind::Markup | SyntaxKind::Code | SyntaxKind::Math => {
            // find the smallest children covering the range
            let mut inner = node
                .children()
                .skip_while(|it| it.range().end <= range.start)
                .take_while(|it| it.range().start < range.end)
                .peekable();
            let start = inner.peek()?.offset();

            // Create a synthetic (mock) syntax node for fine-grained selection.
            // This is a key part of the functionality, as it allows us to refine
            // the range to the smallest children covering the specified range.
            let new_node =
                SyntaxNode::inner(node.kind(), inner.map(|it| it.get().clone()).collect());
            let sub_range = start..start + new_node.len();
            Some((new_node, sub_range))
        }
        _ => Some((node.get().clone(), node.range())),
    }
}

/// Get the range of the string obtained from trimming in the original string.
fn trim_range(s: &str, mut rng: Range<usize>) -> Range<usize> {
    rng.end = rng.start + s[rng.clone()].trim_end().len();
    rng.start = rng.end - s[rng.clone()].trim_start().len();
    rng
}

/// Get a Markup/Expr/Pattern node from source with minimal span that covering the given range.
fn get_node_cover_range(source: &Source, range: Range<usize>) -> Option<LinkedNode<'_>> {
    let range = range.start..range.end.min(source.lines().len_bytes());
    get_node_cover_range_impl(range, LinkedNode::new(source.root()))
}

fn get_node_cover_range_impl<'a>(
    range: Range<usize>,
    node: LinkedNode<'a>,
) -> Option<LinkedNode<'a>> {
    let node_range = node.range();
    if node_range.start > range.start || node_range.end < range.end {
        return None;
    }
    // First, try to find a child node that covers the range
    for child in node.children() {
        if let Some(res) = get_node_cover_range_impl(range.clone(), child) {
            return Some(res);
        }
    }

    // If no child covers the range, check if this node covers it
    (node.is::<Markup>() || node.is::<Code>() || node.is::<Expr>() || node.is::<Pattern>())
        .then_some(node)
}

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, assert_snapshot};

    use super::*;

    #[test]
    fn range_ast_keeps_the_smallest_covering_node() {
        let source = Source::detached("text");
        let ast = format_range_ast(&source, 0..4).unwrap();
        assert_eq!(ast.source_range, 0..4);
        insta::assert_snapshot!(ast.content, @r#"Text: "text""#);
    }

    fn test(content: &str, lc_range: Range<(usize, usize)>) -> RangeResult {
        let source = Source::detached(content);
        let range = source
            .lines()
            .line_column_to_byte(lc_range.start.0, lc_range.start.1)
            .unwrap()
            ..source
                .lines()
                .line_column_to_byte(lc_range.end.0, lc_range.end.1)
                .unwrap();

        let t = Typstyle::default();
        t.format_source_range(source, range).unwrap()
    }

    #[test]
    fn cover_markup() {
        let res = test(
            "
#(1+1)
#(2+2)
#(3+3)",
            (1, 1)..(2, 2),
        );

        assert_debug_snapshot!(res.source_range, @"2..14");
        assert_snapshot!(res.content, @r"
        (1 + 1)
        #(2 + 2)
        ");
    }

    #[test]
    fn cover_markup_empty() {
        let res = test(
            "
#(1+1)
#(2+2)",
            (1, 1)..(1, 1),
        );

        assert_debug_snapshot!(res.source_range, @"2..7");
        assert_snapshot!(res.content, @"(1 + 1)");
    }

    #[test]
    fn cover_markup_empty2() {
        let res = test(" a   b ", (0, 3)..(0, 3));

        assert_debug_snapshot!(res.source_range, @"3..3");
        assert_snapshot!(res.content, @"");
    }

    #[test]
    fn cover_code() {
        let res = test(
            r#"""
#{
("1"+"1")
("2"+"2")
("3"+"3")
}"""#,
            (2, 2)..(3, 3),
        );

        assert_debug_snapshot!(res.source_range, @"6..25");
        assert_snapshot!(res.content, @r#"
        ("1" + "1")
        ("2" + "2")
        "#);
    }

    #[test]
    fn cover_code_empty() {
        let res = test(
            r#"""
#{
("1"+"1")
("2"+"2")
}"""#,
            (2, 2)..(2, 2),
        );

        assert_debug_snapshot!(res.source_range, @"7..10");
        assert_snapshot!(res.content, @r#""1""#);
    }

    #[test]
    fn cover_math() {
        let res = test(
            r#"""$
sin( x )
cos( y )
tan( z )
$"""#,
            (2, 2)..(3, 3),
        );

        assert_debug_snapshot!(res.source_range, @"13..30");
        assert_snapshot!(res.content, @r"
        cos(y)
        tan(z)
        ");
    }
}
