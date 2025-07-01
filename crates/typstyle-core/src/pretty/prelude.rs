pub use pretty::{Arena, DocAllocator, DocBuilder};

pub use super::doc_ext::*;
pub use crate::prettyless::*;

pub type ArenaDoc<'a> = DocBuilder<'a, Arena<'a>>;
