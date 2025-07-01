//! The `prettyless` module
//!
//! Adapted from original [Marwes/pretty.rs](https://github.com/Marwes/pretty.rs).
//! Refactored and extended for:
//!   - clearer implementation and better documentation
//!   - quick union implementation, which only fit the first line

mod render;

use std::{fmt, io};

use pretty::{Doc, DocPtr, FmtWrite, IoWrite, Render};

pub struct PrettyFmt<'a, 'd, T, A>
where
    A: 'a,
    T: DocPtr<'a, A> + 'a,
{
    doc: &'d Doc<'a, T, A>,
    width: usize,
}

impl<'a, T, A> fmt::Display for PrettyFmt<'a, '_, T, A>
where
    T: DocPtr<'a, A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.doc.renderless_fmt(self.width, f)
    }
}

pub trait Prettyless<'a, T, A>
where
    A: 'a,
    T: DocPtr<'a, A> + 'a,
{
    fn renderless<W>(&self, width: usize, out: &mut W) -> io::Result<()>
    where
        W: io::Write;

    fn renderless_fmt<W>(&self, width: usize, out: &mut W) -> fmt::Result
    where
        W: fmt::Write;

    fn renderless_raw<W>(&self, width: usize, out: &mut W) -> Result<(), W::Error>
    where
        W: Render;

    fn prettyless<'d>(&'d self, width: usize) -> PrettyFmt<'a, 'd, T, A>;
}

impl<'a, T, A> Prettyless<'a, T, A> for Doc<'a, T, A>
where
    A: 'a,
    T: DocPtr<'a, A> + 'a,
{
    /// Writes a rendered document to a `std::io::Write` object.
    fn renderless<W>(&self, width: usize, out: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.renderless_raw(width, &mut IoWrite::new(out))
    }

    /// Writes a rendered document to a `std::fmt::Write` object.
    fn renderless_fmt<W>(&self, width: usize, out: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        self.renderless_raw(width, &mut FmtWrite::new(out))
    }

    /// Writes a rendered document to a `RenderAnnotated<A>` object.
    fn renderless_raw<W>(&self, width: usize, out: &mut W) -> Result<(), W::Error>
    where
        W: Render,
    {
        render::print_doc(self, width, out)
    }

    /// Returns a value which implements `std::fmt::Display`
    fn prettyless<'d>(&'d self, width: usize) -> PrettyFmt<'a, 'd, T, A> {
        PrettyFmt { doc: self, width }
    }
}
