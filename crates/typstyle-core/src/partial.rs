use std::ops::Range;

use typst_syntax::{
    ast::{Expr, Markup, Pattern},
    LinkedNode, Source, Span, SyntaxKind,
};

use crate::{
    pretty::{Context, Mode},
    utils, AttrStore, Error, PrettyPrinter, Typstyle,
};

impl Typstyle {
    /// Format the node with minimal span that covering the given range.
    pub fn format_source_range(
        &self,
        source: &Source,
        utf8_range: Range<usize>,
    ) -> Result<(Range<usize>, String), Error> {
        // Trim the give range to ensure no space aside.
        let range = utils::trim_range(source.text(), utf8_range);

        let Some((node, mode)) =
            get_node_cover_range(source, range.clone()).filter(|(node, _)| !node.erroneous())
        else {
            return Err(Error::SyntaxError);
        };

        let attrs = AttrStore::new(node.get()); // Here we only compute the attributes of that subtree.
        let printer = PrettyPrinter::new(self.config.clone(), attrs);
        let ctx = Context::default().with_mode(mode);
        let doc = if let Some(markup) = node.cast() {
            printer.convert_markup(ctx, markup)
        } else if let Some(expr) = node.cast() {
            printer.convert_expr(ctx, expr)
        } else if let Some(pattern) = node.cast() {
            printer.convert_pattern(ctx, pattern)
        } else {
            return Err(Error::SyntaxError);
        };
        // Infer indent from context.
        let indent = utils::count_spaces_after_last_newline(source.text(), range.start);
        let res = doc
            .nest(indent as isize)
            .print(self.config.max_width)
            .to_string();
        Ok((node.range(), res))
    }
}

/// Get a Markup/Expr/Pattern node from source with minimal span that covering the given range.
fn get_node_cover_range(source: &Source, range: Range<usize>) -> Option<(LinkedNode, Mode)> {
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
