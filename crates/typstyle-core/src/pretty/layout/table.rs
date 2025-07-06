use crate::pretty::prelude::*;

pub struct TableCollector<'a> {
    arena: &'a Arena<'a>,
    /// When columns == 0, we will not reflow cells.
    columns: usize,

    /// The rows of the table. Each row is either a list of cells, a block, or a comment.
    rows: Vec<Row<'a>>,
    /// A buffer for cell docs in the current row.
    current_row_cells: Vec<ArenaDoc<'a>>,
}

enum Row<'a> {
    Cells {
        /// The combined doc of the cells in this row.
        doc: ArenaDoc<'a>,
        /// Whether an additional line break can be added after this row.
        auto_break: bool,
    },
    /// A blocky arg that occupies the entire row.
    Block(ArenaDoc<'a>),
    Comment(ArenaDoc<'a>),
    Linebreak,
}

impl<'a> TableCollector<'a> {
    pub fn new(arena: &'a Arena<'a>, columns: usize) -> Self {
        Self {
            columns,
            rows: vec![],
            current_row_cells: Vec::with_capacity(columns.max(2)),
            arena,
        }
    }

    pub fn push_cell(&mut self, doc: ArenaDoc<'a>) {
        self.current_row_cells.push(doc);
        if self.current_row_cells.len() == self.columns {
            self.flush_cells();
        }
    }

    pub fn push_row(&mut self, doc: ArenaDoc<'a>) {
        self.flush_cells();
        self.rows.push(Row::Block(doc));
    }

    pub fn push_comment(&mut self, doc: ArenaDoc<'a>) {
        self.flush_cells();
        self.disable_last_auto_break();
        self.rows.push(Row::Comment(doc));
    }

    pub fn push_newline(&mut self, n: usize) {
        if n == 1 && self.columns == 0 || n > 1 {
            self.flush_cells();
        }
        if n > 1 {
            self.disable_last_auto_break();
            self.rows.push(Row::Linebreak);
        }
    }

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

    fn disable_last_auto_break(&mut self) {
        if let Some(Row::Cells { auto_break, .. }) = self.rows.last_mut() {
            *auto_break = false;
        }
    }

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
            self.arena.hardline(),
        )
    }
}
