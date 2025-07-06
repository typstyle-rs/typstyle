pub use prettyless::{Arena, DocAllocator, DocBuilder};

pub type ArenaDoc<'a> = DocBuilder<'a, Arena<'a>>;
