pub mod attr;
pub mod ext;
pub mod liteval;
pub mod partial;
pub mod pretty;

mod config;
mod utils;

pub use attr::AttrStore;
pub use config::Config;
use pretty::{PrettyPrinter, prelude::*};
use thiserror::Error;
use typst_syntax::{Source, SyntaxNode};

use crate::utils::indent_4_to_2;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The document has syntax errors")]
    SyntaxError,
    #[error("An error occurred while rendering the document")]
    RenderError,
}

/// Main struct for Typst formatting.
#[derive(Debug, Clone, Default)]
pub struct Typstyle {
    config: Config,
}

impl Typstyle {
    /// Creates a new `Typstyle` with the given style configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Prepares a text string for formatting.
    pub fn format_text(&self, text: impl Into<String>) -> Formatter<'_> {
        // We should ensure that the source tree is spanned.
        self.format_source(Source::detached(text.into()))
    }

    /// Prepares a source for formatting.
    pub fn format_source(&self, source: Source) -> Formatter<'_> {
        Formatter::new(self.config.clone(), source)
    }
}

/// Handles the formatting of a specific Typst source.
pub struct Formatter<'a> {
    source: Source,
    printer: PrettyPrinter<'a>,
}

impl<'a> Formatter<'a> {
    fn new(config: Config, source: Source) -> Self {
        let attr_store = AttrStore::new(source.root());
        let printer = PrettyPrinter::new(config, attr_store);
        Self { source, printer }
    }

    /// Renders the document's pretty IR.
    pub fn render_ir(&'a self) -> Result<String, Error> {
        let doc = self.build_doc()?;
        Ok(indent_4_to_2(&format!("{doc:#?}")))
    }

    /// Renders the formatted document to a string.
    pub fn render(&'a self) -> Result<String, Error> {
        let doc = self.build_doc()?;
        let mut buf = String::new();
        doc.render_fmt(self.printer.config().max_width, &mut buf)
            .map_err(|_| Error::RenderError)?;
        let result = utils::strip_trailing_whitespace(&buf);
        Ok(result)
    }

    fn build_doc(&'a self) -> Result<ArenaDoc<'a>, Error> {
        let root = self.source.root();
        if root.erroneous() {
            return Err(Error::SyntaxError);
        }
        let markup = root.cast().unwrap();
        let doc = self.printer.convert_markup(Default::default(), markup);
        Ok(doc)
    }
}

/// Formats a `SyntaxNode` as a debug AST string with 2-space indentation.
pub fn format_ast(root: &SyntaxNode) -> String {
    indent_4_to_2(&format!("{root:#?}"))
}

/// A mapping entry from source byte range to output text byte range.
#[cfg(feature = "mapping")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SpanMapping {
    pub src_start: usize,
    pub src_end: usize,
    pub out_start: usize,
    pub out_end: usize,
}

/// Formats a `SyntaxNode` as a debug AST string and returns span mappings
/// for leaf nodes (nodes with no children), mapping source byte ranges
/// to output text positions.
///
/// Uses a custom formatter that generates output identical to
/// `indent_4_to_2(&format!("{root:#?}"))` while directly recording
/// each leaf node's position in the output buffer — no `find` needed.
#[cfg(feature = "mapping")]
pub fn format_ast_with_mapping(root: &SyntaxNode) -> (String, Vec<SpanMapping>) {
    let mut buf = String::new();
    let mut mappings = Vec::new();
    format_ast_debug(root, &mut buf, &mut mappings, 0, 0);
    (buf, mappings)
}

/// Custom AST debug formatter that produces output identical to
/// `indent_4_to_2(&format!("{node:#?}"))` (2-space indentation).
///
/// For leaf nodes (no children), records the exact output range as a SpanMapping.
/// For inner nodes, recursively formats children with increased indentation.
///
/// `src_offset` tracks the byte offset of `node` in the original source text.
#[cfg(feature = "mapping")]
fn format_ast_debug(
    node: &SyntaxNode,
    buf: &mut String,
    mappings: &mut Vec<SpanMapping>,
    indent: usize,
    src_offset: usize,
) {
    use std::fmt::Write;

    let children: Vec<&SyntaxNode> = node.children().collect();
    if children.is_empty() {
        // Leaf node (or error node): write its Debug repr and record mapping
        let out_start = buf.len();
        // Use the node's own Debug impl (non-pretty, since leaves have no nesting)
        write!(buf, "{node:?}").unwrap();
        let out_end = buf.len();
        let node_len = node.len();
        if node_len > 0 {
            mappings.push(SpanMapping {
                src_start: src_offset,
                src_end: src_offset + node_len,
                out_start,
                out_end,
            });
        }
    } else {
        // Inner node: "{Kind}: {len} [\n  child1,\n  child2,\n]"
        write!(buf, "{:?}: {}", node.kind(), node.len()).unwrap();
        buf.push_str(" [\n");
        let child_indent = indent + 2;
        let mut child_offset = src_offset;
        for child in &children {
            // Write indentation
            for _ in 0..child_indent {
                buf.push(' ');
            }
            format_ast_debug(child, buf, mappings, child_indent, child_offset);
            buf.push_str(",\n");
            child_offset += child.len();
        }
        // Closing bracket at parent indent level
        for _ in 0..indent {
            buf.push(' ');
        }
        buf.push(']');
    }
}

#[cfg(all(test, feature = "mapping"))]
mod ast_debug_tests {
    use super::*;

    fn assert_format_matches(src: &str) {
        let source = Source::detached(src.to_string());
        let root = source.root();
        let expected = indent_4_to_2(&format!("{root:#?}"));
        let (actual, mappings) = format_ast_with_mapping(root);
        assert_eq!(
            actual, expected,
            "Custom formatter output differs from reference"
        );
        // Verify mappings are non-empty and ordered
        assert!(!mappings.is_empty(), "Expected non-empty mappings");
        for w in mappings.windows(2) {
            assert!(
                w[0].out_end <= w[1].out_start,
                "Mapping out ranges overlap: {:?} and {:?}",
                w[0],
                w[1]
            );
            assert!(
                w[0].src_start <= w[1].src_start,
                "Mapping src ranges not ordered: {:?} and {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn test_simple_text() {
        assert_format_matches("Hello *world*");
    }

    #[test]
    fn test_let_binding() {
        assert_format_matches("#let x = 1\nHello");
    }

    #[test]
    fn test_nested_function_call() {
        assert_format_matches(
            r#"#figure(table(
  columns: 2,
  [Benchmark A], [100],
  [Benchmark B], [200],
))"#,
        );
    }

    #[test]
    fn test_empty_document() {
        let source = Source::detached("".to_string());
        let root = source.root();
        let expected = indent_4_to_2(&format!("{root:#?}"));
        let (actual, _) = format_ast_with_mapping(root);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_multiline_with_special_chars() {
        assert_format_matches("Hello\n\"world\"\ttab\\slash");
    }

    #[test]
    fn test_math_mode() {
        assert_format_matches("$x + y = z$");
    }

    #[test]
    fn test_code_block() {
        assert_format_matches("#{\n  let a = 1\n  let b = 2\n}");
    }

    #[test]
    fn test_mapping_coverage() {
        // Verify every leaf gets a mapping
        let src = "#let x = 1\nHello";
        let source = Source::detached(src.to_string());
        let root = source.root();
        let (output, mappings) = format_ast_with_mapping(root);

        // Count leaves
        fn count_leaves(node: &SyntaxNode) -> usize {
            let children: Vec<_> = node.children().collect();
            if children.is_empty() {
                if node.len() > 0 { 1 } else { 0 }
            } else {
                children.iter().map(|c| count_leaves(c)).sum()
            }
        }
        let leaf_count = count_leaves(root);
        assert_eq!(
            mappings.len(),
            leaf_count,
            "Expected one mapping per non-empty leaf, got {} mappings for {} leaves",
            mappings.len(),
            leaf_count,
        );

        // Verify each mapping points to valid content
        for m in &mappings {
            assert!(
                m.out_end <= output.len(),
                "Mapping out_end exceeds output length"
            );
            assert!(
                m.src_end <= src.len(),
                "Mapping src_end exceeds source length"
            );
        }
    }
}
