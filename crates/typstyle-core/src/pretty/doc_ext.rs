use pretty::*;

pub trait DocExt<'a, D, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
{
    fn repeat_n(self, n: usize) -> Self;
}

impl<'a, D, A> DocExt<'a, D, A> for DocBuilder<'a, D, A>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
    DocBuilder<'a, D, A>: Clone,
{
    fn repeat_n(self, n: usize) -> Self {
        let mut doc = self.0.nil();
        for _ in 0..n {
            doc = doc.append(self.clone());
        }
        doc
    }
}

pub trait AllocExt<'a, A> {
    fn spaces(&'a self, n: usize) -> DocBuilder<'a, Self, A>
    where
        Self: DocAllocator<'a, A>;
}

impl<'a, T, A> AllocExt<'a, A> for T
where
    T: DocAllocator<'a, A>,
    A: 'a,
{
    fn spaces(&'a self, n: usize) -> DocBuilder<'a, Self, A> {
        static SPACES: &str =
            "                                                                                ";

        if n == 0 {
            self.nil()
        } else if n <= SPACES.len() {
            self.text(&SPACES[..n])
        } else {
            let mut doc = self.nil();
            let mut remaining = n;
            while remaining != 0 {
                let i = SPACES.len().min(remaining);
                remaining -= i;
                doc = doc.append(self.text(&SPACES[..i]))
            }
            doc
        }
    }
}

pub struct BadFlatten;

pub trait Flatten<'a> {
    fn flatten(self) -> DocBuilder<'a, Arena<'a>>;

    fn try_flatten(self) -> Result<DocBuilder<'a, Arena<'a>>, BadFlatten>;
}

impl<'a> Flatten<'a> for DocBuilder<'a, Arena<'a>> {
    fn flatten(self) -> DocBuilder<'a, Arena<'a>> {
        let DocBuilder(alloc, doc) = self;
        match *doc {
            Doc::Append(a, b) => a.pretty(alloc).flatten().append(b.pretty(alloc).flatten()),
            Doc::Group(g) => g.pretty(alloc).flatten(),
            Doc::FlatAlt(_, b) => b.pretty(alloc).flatten(),
            Doc::Nest(_, b) => b.pretty(alloc).flatten(),
            Doc::Hardline => DocBuilder(alloc, Doc::Fail.into()),
            Doc::Union(a, _) => a.pretty(alloc).flatten(),
            _ => DocBuilder(alloc, doc),
        }
    }

    fn try_flatten(self) -> Result<DocBuilder<'a, Arena<'a>>, BadFlatten> {
        let DocBuilder(alloc, doc) = self;
        match *doc {
            Doc::Append(a, b) => Ok(a.pretty(alloc).flatten().append(b.pretty(alloc).flatten())),
            Doc::Group(g) => g.pretty(alloc).try_flatten(),
            Doc::FlatAlt(_, b) => b.pretty(alloc).try_flatten(),
            Doc::Nest(_, b) => b.pretty(alloc).try_flatten(),
            Doc::Hardline => Err(BadFlatten),
            Doc::Union(a, _) => a.pretty(alloc).try_flatten(),
            _ => Ok(DocBuilder(alloc, doc)),
        }
    }
}
