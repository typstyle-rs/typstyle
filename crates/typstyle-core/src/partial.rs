use std::{borrow::Cow, ops::Range};

use itertools::Itertools;
use typst_syntax::{ast::*, LinkedNode, Source, Span, SyntaxKind, SyntaxNode};

use crate::{pretty::Mode, utils, AttrStore, Error, PrettyPrinter, Typstyle};

/// Result of a range operation (formatting, IR generation, etc.).
#[derive(Debug, Clone)]
pub struct RangeResult {
    /// The actual source range that was processed (may be larger than requested to include complete nodes)
    pub source_range: Range<usize>,
    /// The processed content for the range (formatted text, IR, AST, etc.)
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
    /// Format the node with minimal span that covering the given range.
    ///
    /// This function finds the smallest complete syntax node that encompasses the given range
    /// and formats it. The actual formatted range may be larger than the input range to ensure
    /// syntactically valid formatting.
    ///
    /// # Arguments
    /// - `source` - The source code to format
    /// - `utf8_range` - The UTF-8 byte range to format
    ///
    /// # Returns
    /// A `RangeResult` containing the formatted text and metadata about the operation.
    pub fn format_source_range(
        &self,
        source: Source,
        utf8_range: Range<usize>,
    ) -> Result<RangeResult, Error> {
        let trimmed_range = utils::trim_range(source.text(), utf8_range);
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

    /// Get the pretty IR representation of the node with minimal span that covers the given range.
    ///
    /// This function finds the smallest complete syntax node that encompasses the given range
    /// and returns its pretty IR representation.
    ///
    /// # Arguments
    /// - `source` - The source code to analyze
    /// - `utf8_range` - The UTF-8 byte range to analyze
    ///
    /// # Returns
    /// A `RangeResult` containing the IR representation and metadata about the operation.
    pub fn format_source_range_ir(
        &self,
        source: Source,
        utf8_range: Range<usize>,
    ) -> Result<RangeResult, Error> {
        let trimmed_range = utils::trim_range(source.text(), utf8_range);
        let (node, mode) = get_node_and_mode_for_range(&source, trimmed_range.clone())?;

        let Some((node, node_range)) = refine_node_range(node, trimmed_range.clone()) else {
            return Ok(RangeResult::empty(trimmed_range.start)); // No edit
        };

        let attrs = AttrStore::new(&node);
        let printer = PrettyPrinter::new(self.config.clone(), attrs);
        let doc = printer.try_convert_with_mode(&node, mode)?;

        let ir = format!("{doc:#?}");

        Ok(RangeResult {
            source_range: node_range,
            content: ir,
        })
    }
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

pub fn get_node_for_range(
    source: &Source,
    utf8_range: Range<usize>,
) -> Result<LinkedNode<'_>, Error> {
    // Trim the given range to ensure no space aside.
    let trimmed_range = utils::trim_range(source.text(), utf8_range);

    get_node_and_mode_for_range(source, trimmed_range).map(|(node, _)| node)
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
    let range = range.start..range.end.min(source.len_bytes());
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
            .line_column_to_byte(lc_range.start.0, lc_range.start.1)
            .unwrap()
            ..source
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
