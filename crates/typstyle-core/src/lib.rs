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

#[cfg(feature = "mapping")]
use crate::ir_mapping::{IrAtomOutput, join_ir_mappings, remap_offsets_for_indent_change};
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
        #[cfg(feature = "mapping")]
        let printer = PrettyPrinter::new_with_source(config, attr_store, source.clone());
        #[cfg(not(feature = "mapping"))]
        let printer = PrettyPrinter::new(config, attr_store);
        Self { source, printer }
    }

    /// Renders the document's pretty IR.
    pub fn render_ir(&'a self) -> Result<String, Error> {
        #[cfg(feature = "mapping")]
        {
            self.render_ir_with_mapping().map(|(ir, _)| ir)
        }

        #[cfg(not(feature = "mapping"))]
        {
            let doc = self.build_doc()?;
            Ok(indent_4_to_2(&format!("{doc:#?}")))
        }
    }

    /// Renders the document's pretty IR and source-to-IR span mappings.
    #[cfg(feature = "mapping")]
    pub fn render_ir_with_mapping(&'a self) -> Result<(String, Vec<SpanMapping>), Error> {
        let doc = self.build_doc()?;
        let src_atoms = self.printer.snapshot_ir_atom_sources();
        let (ir_4, tag_ranges) = prettyless::debug_with_tag_ranges(&doc);

        let mut out_atoms: Vec<IrAtomOutput> = tag_ranges
            .into_iter()
            .map(|r| IrAtomOutput {
                atom_id: r.tag,
                out_start: r.start,
                out_end: r.end,
            })
            .collect();

        let ir_text = remap_offsets_for_indent_change(&ir_4, 4, 2, &mut out_atoms);
        let mapping = join_ir_mappings(&src_atoms, &out_atoms);
        Ok((ir_text, mapping))
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
        #[cfg(feature = "mapping")]
        self.printer.reset_ir_mapping();
        let markup = root.cast().unwrap();
        let doc = self.printer.convert_markup(Default::default(), markup);
        Ok(doc)
    }
}

/// Formats a `SyntaxNode` as a debug AST string with 2-space indentation.
pub fn format_ast(root: &SyntaxNode) -> String {
    indent_4_to_2(&format!("{root:#?}"))
}

#[cfg(feature = "mapping")]
mod ast_mapping;
#[cfg(feature = "mapping")]
pub use ast_mapping::{SpanMapping, format_ast_with_mapping};
#[cfg(feature = "mapping")]
mod ir_mapping;

#[cfg(all(test, feature = "mapping"))]
mod mapping_tests {
    use super::*;

    #[test]
    fn render_ir_with_mapping_ranges_are_valid() {
        let input = "#let x = (1, 2)\nHello *world*";
        let typstyle = Typstyle::default();
        let formatter = typstyle.format_text(input);
        let (ir, mapping) = formatter
            .render_ir_with_mapping()
            .expect("render ir mapping");

        assert!(
            !mapping.is_empty(),
            "expected non-empty source-to-ir mapping"
        );
        for m in mapping {
            assert!(m.src_start <= m.src_end, "invalid source range");
            assert!(m.out_start <= m.out_end, "invalid output range");
            assert!(m.src_end <= input.len(), "source range out of bounds");
            assert!(m.out_end <= ir.len(), "output range out of bounds");
        }
    }

    #[test]
    fn render_ir_with_mapping_preserves_unicode_boundaries() {
        let input = "你好 *世界*";
        let typstyle = Typstyle::default();
        let formatter = typstyle.format_text(input);
        let (_ir, mapping) = formatter
            .render_ir_with_mapping()
            .expect("render ir mapping");

        assert!(!mapping.is_empty(), "expected non-empty mapping");
        let mut saw_non_ascii_slice = false;
        for m in mapping {
            assert!(
                input.is_char_boundary(m.src_start),
                "src_start should be UTF-8 char boundary"
            );
            assert!(
                input.is_char_boundary(m.src_end),
                "src_end should be UTF-8 char boundary"
            );
            let s = &input[m.src_start..m.src_end];
            if !s.is_ascii() {
                saw_non_ascii_slice = true;
            }
        }
        assert!(
            saw_non_ascii_slice,
            "expected at least one mapping over non-ascii source text"
        );
    }
}
