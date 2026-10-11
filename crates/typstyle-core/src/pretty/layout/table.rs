//! Collects table and grid cells into rows aligned by a fixed column count.
//!
//! Comments and blank lines form their own rows instead of being reflowed.

use crate::pretty::prelude::*;

/// Collects table or grid cells and reflows them into rows of a fixed column count.
pub struct TableCollector<'a> {
    /// The arena that the row documents are allocated in.
    arena: &'a Arena<'a>,
    /// When columns == 0, we will not reflow cells.
    columns: usize,

    /// The rows of the table. Each row is either a list of cells, a block, or a comment.
    rows: Vec<Row<'a>>,
    /// A buffer for cell docs in the current row.
    current_row_cells: Vec<ArenaDoc<'a>>,
}

/// A row of the collected table.
enum Row<'a> {
    /// A row of cells joined by commas.
    Cells {
        /// The combined doc of the cells in this row.
        doc: ArenaDoc<'a>,
        /// Whether an additional line break can be added after this row.
        auto_break: bool,
    },
    /// A blocky arg that occupies the entire row.
    Block(ArenaDoc<'a>),
    /// A comment occupying its own row.
    Comment(ArenaDoc<'a>),
    /// A blank line preserved from the source.
    Linebreak,
}

impl<'a> TableCollector<'a> {
    /// Creates a collector that reflows cells into rows of [`Self::columns`] cells.
    ///
    /// With `columns == 0`, cells are not reflowed; rows are only flushed at source
    /// line breaks.
    pub fn new(arena: &'a Arena<'a>, columns: usize) -> Self {
        Self {
            columns,
            rows: vec![],
            current_row_cells: Vec::with_capacity(columns.max(2)),
            arena,
        }
    }

    /// Pushes a cell, flushing the current row once it holds [`Self::columns`] cells.
    pub fn push_cell(&mut self, doc: ArenaDoc<'a>) {
        self.current_row_cells.push(doc);
        if self.current_row_cells.len() == self.columns {
            self.flush_cells();
        }
    }

    /// Pushes a row that occupies an entire line, flushing any pending cells first.
    pub fn push_row(&mut self, doc: ArenaDoc<'a>) {
        self.flush_cells();
        self.rows.push(Row::Block(doc));
    }

    /// Pushes a comment as its own row, suppressing the automatic break after the
    /// previous row.
    pub fn push_comment(&mut self, doc: ArenaDoc<'a>) {
        self.flush_cells();
        self.disable_last_auto_break();
        self.rows.push(Row::Comment(doc));
    }

    /// Records `n` source line breaks between cells.
    ///
    /// A single break flushes the pending cells only when not reflowing
    /// ([`Self::columns`] is 0); multiple breaks also insert a blank line and
    /// suppress the automatic break after the previous row.
    pub fn push_newline(&mut self, n: usize) {
        if n == 1 && self.columns == 0 || n > 1 {
            self.flush_cells();
        }
        if n > 1 {
            self.disable_last_auto_break();
            self.rows.push(Row::Linebreak);
        }
    }

    /// Flushes the pending cells into a row joined by commas and optional line breaks.
    fn flush_cells(&mut self) {
        if !self.current_row_cells.is_empty() {
            self.rows.push(Row::Cells {
                doc: self.arena.intersperse(
                    std::mem::replace(
                        &mut self.current_row_cells,
                        Vec::with_capacity(self.columns.max(2)),
                    ),
                    self.arena.text(",") + self.arena.line(),
                ),
                auto_break: self.columns > 1,
            });
        }
    }

    /// Suppresses the optional line break after the last cells row.
    fn disable_last_auto_break(&mut self) {
        if let Some(Row::Cells { auto_break, .. }) = self.rows.last_mut() {
            *auto_break = false;
        }
    }

    /// Builds the final document, joining rows with hard line breaks and dropping
    /// trailing blank lines.
    pub fn collect(mut self) -> ArenaDoc<'a> {
        self.flush_cells();
        while matches!(self.rows.last(), Some(Row::Linebreak)) {
            self.rows.pop();
        }
        let num_rows = self.rows.len();
        let only_one_row = num_rows == 1;
        self.arena.intersperse(
            self.rows.into_iter().enumerate().map(|(i, row)| match row {
                Row::Cells {
                    mut doc,
                    auto_break,
                } => {
                    doc += if only_one_row {
                        self.arena.text(",").when_group_break()
                    } else {
                        self.arena.text(",")
                    };
                    if i + 1 < num_rows && auto_break {
                        doc += self.arena.line_()
                    }
                    doc.group()
                }
                Row::Block(doc) => doc + self.arena.text(","),
                Row::Comment(doc) => doc,
                Row::Linebreak => self.arena.nil(),
            }),
            self.arena.hard_line(),
        )
    }
}
