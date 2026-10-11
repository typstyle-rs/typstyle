//! Shared imports and aliases used across the pretty-printing modules.

pub use prettyless::{Arena, DocAllocator, DocBuilder};

/// A pretty document allocated in the printer's [`Arena`].
pub type ArenaDoc<'a> = DocBuilder<'a, Arena<'a>>;
