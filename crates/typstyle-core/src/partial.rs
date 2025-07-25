use std::ops::Range;

use typst_syntax::{
    ast::{Expr, Markup, Pattern},
    LinkedNode, Source, Span, SyntaxKind,
};

use crate::{pretty::Mode, utils, AttrStore, Error, PrettyPrinter, Typstyle};

/// Result of a range formatting operation.
#[derive(Debug, Clone)]
pub struct FormatRangeResult {
    /// The actual range that was formatted (may be larger than requested to include complete nodes)
    pub range: Range<usize>,
    /// The formatted text for the range
    pub text: String,
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
    /// A `FormatRangeResult` containing the formatted text and metadata about the operation.
    pub fn format_source_range(
        &self,
        source: Source,
        utf8_range: Range<usize>,
    ) -> Result<FormatRangeResult, Error> {
        let (node, mode) = get_node_and_mode_for_range(&source, utf8_range)?;

        let node_range = node.range();

        let attrs = AttrStore::new(node.get()); // Here we only compute the attributes of that subtree.
        let printer = PrettyPrinter::new(self.config.clone(), attrs);
        let doc = printer.try_convert_with_mode(&node, mode)?;

        // Infer indent from context.
        let indent = utils::count_spaces_after_last_newline(source.text(), node_range.start);
        let text = doc
            .nest(indent as isize)
            .print(self.config.max_width)
            .to_string();

        Ok(FormatRangeResult {
            range: node_range,
            text,
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
    /// A `FormatRangeResult` containing the IR representation and metadata about the operation.
    pub fn format_source_range_ir(
        &self,
        source: Source,
        utf8_range: Range<usize>,
    ) -> Result<FormatRangeResult, Error> {
        let (node, mode) = get_node_and_mode_for_range(&source, utf8_range)?;

        let node_range = node.range();

        let attrs = AttrStore::new(node.get());
        let printer = PrettyPrinter::new(self.config.clone(), attrs);
        let doc = printer.try_convert_with_mode(&node, mode)?;

        let ir = format!("{doc:#?}");

        Ok(FormatRangeResult {
            range: node_range,
            text: ir,
        })
    }
}

pub fn get_node_for_range(
    source: &Source,
    utf8_range: Range<usize>,
) -> Result<LinkedNode<'_>, Error> {
    get_node_and_mode_for_range(source, utf8_range).map(|(node, _)| node)
}

fn get_node_and_mode_for_range(
    source: &Source,
    utf8_range: Range<usize>,
) -> Result<(LinkedNode<'_>, Mode), Error> {
    // Trim the given range to ensure no space aside.
    let trimmed_range = utils::trim_range(source.text(), utf8_range.clone());

    get_node_cover_range(source, trimmed_range.clone())
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
    for child in node.children() {
        if let Some(res) = get_node_cover_range_impl(range.clone(), child, mode) {
            return Some(res);
        }
    }
    let node_range = node.range();
    (node_range.start <= range.start
        && node_range.end >= range.end
        && (node.is::<Markup>() || node.is::<Expr>() || node.is::<Pattern>()))
    .then(|| (node.span(), mode))
    // It returns span to avoid problems with borrowing.
}
