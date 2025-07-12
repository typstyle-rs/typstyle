pub mod attr;
pub mod ext;
pub mod liteval;
pub mod partial;
pub mod pretty;

mod config;
mod utils;

pub use attr::AttrStore;
pub use config::Config;
use pretty::{prelude::*, PrettyPrinter};
use thiserror::Error;
use typst_syntax::Source;

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
    pub fn format_text(&self, text: impl Into<String>) -> Formatter {
        // We should ensure that the source tree is spanned.
        self.format_source(Source::detached(text.into()))
    }

    /// Prepares a source for formatting.
    pub fn format_source(&self, source: Source) -> Formatter {
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
        Ok(format!("{doc:#?}"))
    }

    /// Renders the formatted document to a string.
    pub fn render(&'a self) -> Result<String, Error> {
        let doc = self.build_doc()?;
        let mut buf = String::new();
        doc.render_fmt(self.printer.config().max_width, &mut buf)
            .map_err(|_| Error::RenderError)?;
        let result = utils::strip_trailing_whitespace(&buf);
        let result = utils::strip_trailing_whitespace(&result);
        let result = utils::strip_trailing_whitespace(&result);
        let result = utils::strip_trailing_whitespace(&result);
        let result = utils::strip_trailing_whitespace(&result);
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
