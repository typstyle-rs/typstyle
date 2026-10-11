//! Provides [`FormattedWorld`], which layers formatted sources over a base compilation world.

use anyhow::Result;
use rustc_hash::FxHashMap;
use tinymist_world::{CompilerFeat, CompilerWorld, SourceWorld, vfs::PathResolution};
use typst::{
    Library, World,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, Source},
    text::{Font, FontBook},
    utils::LazyHash,
};

/// Maps file IDs to their formatted sources.
pub type SourceMap = FxHashMap<FileId, Source>;

/// A world that contains the formatted sources.
pub struct FormattedWorld<'a, F: CompilerFeat> {
    /// The base world providing everything except the overridden sources.
    pub(crate) base: &'a CompilerWorld<F>,
    /// The formatted sources that replace the base ones.
    pub(crate) formatted: SourceMap,
}

impl<'a, F: CompilerFeat> FormattedWorld<'a, F> {
    /// Returns the base world the formatted sources are layered on.
    pub fn base(&self) -> &'a CompilerWorld<F> {
        self.base
    }
}

impl<F: CompilerFeat> World for FormattedWorld<'_, F> {
    fn library(&self) -> &LazyHash<Library> {
        self.base.library()
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.base.book()
    }

    fn main(&self) -> FileId {
        self.base.main()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let formatted = &self.formatted;
        formatted
            .get(&id)
            .map(|source| Ok(source.clone()))
            .unwrap_or_else(|| self.base.source(id))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.base.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.base.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.base.today(offset)
    }
}

impl<F: CompilerFeat> SourceWorld for FormattedWorld<'_, F> {
    fn as_world(&self) -> &dyn World {
        self
    }

    fn path_for_id(&self, id: FileId) -> Result<PathResolution, FileError> {
        self.base.path_for_id(id)
    }
}
