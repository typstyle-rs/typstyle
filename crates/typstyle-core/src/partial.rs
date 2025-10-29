//! Range-based formatting and analysis utilities for Typstyle.
//!
//! All byte ranges in this module are specified as UTF-8 offsets.
//! If you have a UTF-16 range or line-column range (as used in LSP or some editors),
//! you must convert it to a UTF-8 byte offset before calling these functions.
//! The `typst_syntax::Source` API provides helpers for converting between line-column,
//! UTF-16, and UTF-8 offsets.

use std::{borrow::Cow, ops::Range};

use itertools::Itertools;
use typst_syntax::{ast::*, LinkedNode, Source, Span, SyntaxKind, SyntaxNode};

use crate::{
    pretty::Mode,
    utils::{self, indent_4_to_2},
    AttrStore, Error, PrettyPrinter, Typstyle,
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

impl Typstyle {
    /// Format the smallest syntax node that fully covers the given byte range.
    ///
    /// The formatted range may be larger than the input to ensure valid syntax.
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
        let trimmed_range = trim_range(source.text(), utf8_range);
        let (node, mode) = get_node_and_mode_for_range(&source, trimmed_range.clone())?;

        let Some((node, node_range)) = refine_node_range(node, trimmed_range.clone()) else {
            return Ok(RangeResult::empty(trimmed_range.start)); // No edit
        };

        let attrs = AttrStore::new(&node); // Here we only compute the attributes of that subtree.
        let printer = PrettyPrinter::new(self.config.clone(), attrs);
        let doc = printer.try_convert_with_mode(&node, mode)?;

        // Infer indent from context.
        let indent = utils::count_spaces_after_last_newline(source.text(), node_range.start);
        let text = doc
            .nest(indent as isize)
            .print(self.config.max_width)
            .to_string();

        Ok(RangeResult {
            source_range: node_range,
            content: text,
        })
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
        let trimmed_range = trim_range(source.text(), utf8_range);
        let (node, mode) = get_node_and_mode_for_range(&source, trimmed_range.clone())?;

        let Some((node, node_range)) = refine_node_range(node, trimmed_range.clone()) else {
            return Ok(RangeResult::empty(trimmed_range.start)); // No edit
        };

        let attrs = AttrStore::new(&node);
        let printer = PrettyPrinter::new(self.config.clone(), attrs);
        let doc = printer.try_convert_with_mode(&node, mode)?;

        let ir = indent_4_to_2(&format!("{doc:#?}"));

        Ok(RangeResult {
            source_range: node_range,
            content: ir,
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
    let node = get_node_for_range(source, utf8_range)?;
    Ok(RangeResult {
        source_range: node.range(),
        content: indent_4_to_2(&format!("{node:#?}")),
    })
}

fn refine_node_range(
    node: LinkedNode,
    range: Range<usize>,
) -> Option<(Cow<SyntaxNode>, Range<usize>)> {
    match node.kind() {
        SyntaxKind::Markup | SyntaxKind::Code | SyntaxKind::Math => {
            // find the smallest children covering the range
            let inner = node
                .children()
                .skip_while(|it| it.range().end <= range.start)
                .take_while(|it| it.range().start < range.end)
                .collect_vec();
            let sub_range = inner.first()?.range().start..inner.last()?.range().end;

            // Create a synthetic (mock) syntax node for fine-grained selection.
            // This is a key part of the functionality, as it allows us to refine
            // the range to the smallest children covering the specified range.
            let new_node = SyntaxNode::inner(
                node.kind(),
                inner.into_iter().map(|it| it.get().clone()).collect(),
            );
            Some((Cow::Owned(new_node), sub_range))
        }
        _ => Some((Cow::Borrowed(node.get()), node.range())),
    }
}

fn get_node_for_range(source: &Source, utf8_range: Range<usize>) -> Result<LinkedNode<'_>, Error> {
    // Trim the given range to ensure no space aside.
    let trimmed_range = trim_range(source.text(), utf8_range);

    get_node_and_mode_for_range(source, trimmed_range).map(|(node, _)| node)
}

/// Get the range of the string obtained from trimming in the original string.
fn trim_range(s: &str, mut rng: Range<usize>) -> Range<usize> {
    rng.end = rng.start + s[rng.clone()].trim_end().len();
    rng.start = rng.end - s[rng.clone()].trim_start().len();
    rng
}

fn get_node_and_mode_for_range(
    source: &Source,
    utf8_range: Range<usize>,
) -> Result<(LinkedNode<'_>, Mode), Error> {
    get_node_cover_range(source, utf8_range)
        .filter(|(node, _)| !node.erroneous())
        .ok_or(Error::SyntaxError)
}

/// Get a Markup/Expr/Pattern node from source with minimal span that covering the given range.
fn get_node_cover_range(source: &Source, range: Range<usize>) -> Option<(LinkedNode<'_>, Mode)> {
    let range = range.start..range.end.min(source.lines().len_bytes());
    get_node_cover_range_impl(range, LinkedNode::new(source.root()), Mode::Markup)
        .and_then(|(span, mode)| source.find(span).map(|node| (node, mode)))
}

fn get_node_cover_range_impl(
    range: Range<usize>,
    node: LinkedNode<'_>,
    mode: Mode,
) -> Option<(Span, Mode)> {
    let mode = match node.kind() {
        SyntaxKind::Markup => Mode::Markup,
        SyntaxKind::CodeBlock => Mode::Code,
        SyntaxKind::Equation => Mode::Math,
        _ => mode,
    };

    // First, try to find a child node that covers the range
    for child in node.children() {
        if let Some(res) = get_node_cover_range_impl(range.clone(), child, mode) {
            return Some(res);
        }
    }

    // If no child covers the range, check if this node covers it
    let node_range = node.range();
    (node_range.start <= range.start
        && node_range.end >= range.end
        && (node.is::<Markup>()
            || node.is::<Code>()
            || node.is::<Math>()
            || node.is::<Expr>()
            || node.is::<Pattern>()))
    .then(|| (node.span(), mode))
    // It returns span to avoid problems with borrowing.
}

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, assert_snapshot};

    use super::*;

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

        assert_debug_snapshot!(res.source_range, @"2..5");
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
