//! Adapted from https://github.com/Marwes/pretty.rs/blob/master/src/render.rs.

use pretty::{Doc, DocPtr, Render};

pub fn print_doc<'a, W, T, A>(
    doc: &Doc<'a, T, A>,
    width: usize,
    out: &mut W,
) -> Result<(), W::Error>
where
    T: DocPtr<'a, A> + 'a,
    W: Render,
{
    let temp_arena = &typed_arena::Arena::new();
    Printer {
        pos: 0,
        cmds: vec![Cmd {
            width,
            indent: 0,
            mode: Mode::Break,
            doc,
        }],
        fit_docs: vec![],
        temp_arena,
    }
    .print_to(0, false, out)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Mode {
    Break,
    Flat,
}

struct Cmd<'d, 'a, T, A>
where
    T: DocPtr<'a, A> + 'a,
{
    width: usize,
    indent: usize,
    mode: Mode,
    doc: &'d Doc<'a, T, A>,
}

struct FitCmd<'d, 'a, T, A>
where
    T: DocPtr<'a, A> + 'a,
{
    width: usize,
    mode: Mode,
    doc: &'d Doc<'a, T, A>,
}

struct Printer<'d, 'a, T, A>
where
    T: DocPtr<'a, A> + 'a,
{
    pos: usize,
    cmds: Vec<Cmd<'d, 'a, T, A>>,
    fit_docs: Vec<FitCmd<'d, 'a, T, A>>,
    temp_arena: &'d typed_arena::Arena<T>,
}

impl<'d, 'a, T, A> Printer<'d, 'a, T, A>
where
    T: DocPtr<'a, A> + 'a,
{
    fn print_to<W>(&mut self, top: usize, quick: bool, out: &mut W) -> Result<bool, W::Error>
    where
        W: Render,
    {
        let mut fits = true;
        let mut on_first_line = true;
        while self.cmds.len() > top {
            // Pop the next command
            let mut cmd = self.cmds.pop().unwrap();

            // Drill down until we hit a leaf or emit something
            loop {
                let Cmd {
                    width,
                    indent,
                    mode,
                    doc,
                } = cmd;
                match *doc {
                    Doc::Nil => break,
                    Doc::Fail => return Err(out.fail_doc()),

                    Doc::OwnedText(ref s) => {
                        out.write_str_all(s)?;
                        self.pos += s.len();
                        fits &= self.pos <= width || quick && !on_first_line;
                        break;
                    }
                    Doc::BorrowedText(s) => {
                        out.write_str_all(s)?;
                        self.pos += s.len();
                        fits &= self.pos <= width || quick && !on_first_line;
                        break;
                    }
                    Doc::SmallText(ref s) => {
                        out.write_str_all(s)?;
                        self.pos += s.len();
                        fits &= self.pos <= width || quick && !on_first_line;
                        break;
                    }

                    Doc::RenderLen(len, ref inner) => {
                        // inner must be a text node
                        let str = match **inner {
                            Doc::OwnedText(ref s) => s,
                            Doc::BorrowedText(s) => s,
                            Doc::SmallText(ref s) => s,
                            _ => unreachable!(),
                        };
                        out.write_str_all(str)?;
                        self.pos += len;
                        fits &= self.pos <= width || quick && !on_first_line;
                        break;
                    }

                    Doc::Hardline => {
                        // The next document may have different indentation so we should use it if
                        // we can
                        on_first_line = false;
                        fits &= mode == Mode::Break;
                        if let Some(next) = self.cmds.pop() {
                            write_newline(next.indent, out)?;
                            self.pos = next.indent;
                            cmd = next;
                        } else {
                            write_newline(indent, out)?;
                            self.pos = indent;
                            break;
                        }
                    }

                    Doc::Append(ref left, ref right) => {
                        // Push children in reverse so we process ldoc before rdoc
                        cmd.doc = append_docs2(left, right, |doc| {
                            self.cmds.push(Cmd {
                                width,
                                indent,
                                mode,
                                doc,
                            })
                        });
                    }
                    Doc::Nest(offset, ref inner) => {
                        cmd.indent = indent.saturating_add_signed(offset);
                        cmd.doc = inner;
                    }

                    Doc::Group(ref inner) => {
                        if mode == Mode::Break
                            && self.fitting(inner, self.pos, width, indent, Mode::Flat)
                        {
                            cmd.mode = Mode::Flat;
                        }
                        cmd.doc = inner;
                    }
                    Doc::FlatAlt(ref break_doc, ref flat_doc) => {
                        cmd.doc = match mode {
                            Mode::Break => break_doc,
                            Mode::Flat => flat_doc,
                        };
                    }
                    Doc::Union(ref left, ref right) => {
                        if mode == Mode::Flat
                            || self.fitting(left, self.pos, width, indent, Mode::Break)
                        {
                            cmd.doc = left;
                        } else {
                            cmd.doc = right;
                        }
                    }

                    Doc::Column(ref f) => {
                        cmd.doc = self.temp_arena.alloc(f(self.pos));
                    }
                    Doc::Nesting(ref f) => {
                        cmd.doc = self.temp_arena.alloc(f(indent));
                    }

                    Doc::Annotated(_, _) => {} // Ignore
                }
            }
        }
        Ok(fits)
    }

    fn fitting(
        &mut self,
        next: &'d Doc<'a, T, A>,
        mut pos: usize,
        width: usize,
        indent: usize,
        mode: Mode,
    ) -> bool {
        // We start in "flat" mode and may fall back to "break" mode when backtracking.
        let mut cmd_bottom = self.cmds.len();

        // fit_docs is our work‐stack for documents to check in flat mode.
        self.fit_docs.clear();
        self.fit_docs.push(FitCmd {
            width,
            mode,
            doc: next,
        });

        // As long as we have either flat‐stack items or break commands to try...
        while cmd_bottom > 0 || !self.fit_docs.is_empty() {
            // Pop the next doc to inspect, or backtrack to bcmds in break mode.
            let FitCmd {
                width,
                mode,
                mut doc,
            } = self.fit_docs.pop().unwrap_or_else(|| {
                cmd_bottom -= 1;
                FitCmd {
                    width,
                    mode: Mode::Break,
                    doc: self.cmds[cmd_bottom].doc,
                }
            });

            // Drill into this doc until we either bail or consume a leaf.
            loop {
                match *doc {
                    Doc::Nil => break,
                    Doc::Fail => return false,

                    Doc::OwnedText(ref s) => {
                        pos += s.len();
                        if pos > width {
                            return false;
                        }
                        break;
                    }
                    Doc::BorrowedText(s) => {
                        pos += s.len();
                        if pos > width {
                            return false;
                        }
                        break;
                    }
                    Doc::SmallText(ref s) => {
                        pos += s.len();
                        if pos > width {
                            return false;
                        }
                        break;
                    }
                    Doc::RenderLen(len, _) => {
                        pos += len;
                        if pos > width {
                            return false;
                        }
                        break;
                    }

                    Doc::Hardline => {
                        // A hardline only “fits” in break mode.
                        return mode == Mode::Break;
                    }

                    Doc::Append(ref left, ref right) => {
                        // Push r then l so we process l first.
                        doc = append_docs2(left, right, |doc| {
                            self.fit_docs.push(FitCmd { width, mode, doc })
                        });
                    }

                    Doc::FlatAlt(ref break_doc, ref flat_doc) => {
                        // Select branch based on current mode.
                        doc = if mode == Mode::Break {
                            break_doc
                        } else {
                            flat_doc
                        };
                    }

                    Doc::Nest(_, ref inner)
                    | Doc::Group(ref inner)
                    | Doc::Annotated(_, ref inner)
                    | Doc::Union(_, ref inner) => {
                        doc = inner;
                    }

                    Doc::Column(ref f) => {
                        doc = self.temp_arena.alloc(f(pos));
                    }

                    Doc::Nesting(ref f) => {
                        doc = self.temp_arena.alloc(f(indent));
                    }
                }
            }
        }

        // If we've exhausted both fcmds and break_idx, everything fit.
        true
    }
}

fn append_docs2<'a, 'd, T, A>(
    ldoc: &'d Doc<'a, T, A>,
    rdoc: &'d Doc<'a, T, A>,
    mut consumer: impl FnMut(&'d Doc<'a, T, A>),
) -> &'d Doc<'a, T, A>
where
    T: DocPtr<'a, A>,
{
    let d = append_docs(rdoc, &mut consumer);
    consumer(d);
    append_docs(ldoc, &mut consumer)
}

fn append_docs<'a, 'd, T, A>(
    mut doc: &'d Doc<'a, T, A>,
    consumer: &mut impl FnMut(&'d Doc<'a, T, A>),
) -> &'d Doc<'a, T, A>
where
    T: DocPtr<'a, A>,
{
    loop {
        // Since appended documents often appear in sequence on the left side we
        // gain a slight performance increase by batching these pushes (avoiding
        // to push and directly pop `Append` documents)
        match doc {
            Doc::Append(l, r) => {
                consumer(r);
                doc = l;
            }
            _ => return doc,
        }
    }
}

fn write_newline<W>(ind: usize, out: &mut W) -> Result<(), W::Error>
where
    W: Render,
{
    out.write_str_all("\n")?;
    write_spaces(ind, out)
}

fn write_spaces<W>(spaces: usize, out: &mut W) -> Result<(), W::Error>
where
    W: Render,
{
    let mut inserted = 0;
    while inserted < spaces {
        let insert = SPACES.len().min(spaces - inserted);
        inserted += out.write_str(&SPACES[..insert])?;
    }

    Ok(())
}

pub(crate) const SPACES: &str =
    "                                                                                ";
