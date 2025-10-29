use crate::pretty::{PrettyPrinter, prelude::*};

/// An item in the flow. A space is added only when the item before and the item after both allow it.
pub struct FlowItem<'a>(pub Option<FlowItemRepr<'a>>);

pub struct FlowItemRepr<'a> {
    pub doc: ArenaDoc<'a>,
    /// Whether a space can be added before the doc.
    pub space_before: bool,
    /// Whether a space can be added after the doc.
    pub space_after: bool,
}

impl<'a> FlowItem<'a> {
    pub fn new(doc: ArenaDoc<'a>, space_before: bool, space_after: bool) -> Self {
        Self(Some(FlowItemRepr {
            doc,
            space_before,
            space_after,
        }))
    }

    /// Create an item that allows spaces before and after.
    pub fn spaced(doc: ArenaDoc<'a>) -> Self {
        Self::new(doc, true, true)
    }

    /// Create an item that allows space before.
    pub fn spaced_before(doc: ArenaDoc<'a>, space_after: bool) -> Self {
        Self::new(doc, true, space_after)
    }

    /// Create an item that disallows space before but allows space after.
    pub fn tight_spaced(doc: ArenaDoc<'a>) -> Self {
        Self::new(doc, false, true)
    }

    /// Create an item that disallows space after but allows space before.
    pub fn spaced_tight(doc: ArenaDoc<'a>) -> Self {
        Self::new(doc, true, false)
    }

    /// Create an item that disallows space before and after.
    pub fn tight(doc: ArenaDoc<'a>) -> Self {
        Self::new(doc, false, false)
    }

    pub const fn none() -> Self {
        Self(None)
    }
}

pub struct FlowStylist<'a> {
    printer: &'a PrettyPrinter<'a>,
    doc: ArenaDoc<'a>,
    space_after: bool,
    at_line_start: bool,
}

impl<'a> FlowStylist<'a> {
    pub fn new(printer: &'a PrettyPrinter<'a>) -> Self {
        Self {
            doc: printer.arena.nil(),
            printer,
            space_after: false,
            at_line_start: true,
        }
    }

    pub fn push_comment(&mut self, doc: ArenaDoc<'a>, is_block: bool) {
        if is_block {
            self.push_doc(doc, true, true);
        } else {
            if !self.at_line_start {
                self.space_after = true;
            }
            self.push_doc(doc, true, false);
        }
    }

    pub fn push_doc(&mut self, doc: ArenaDoc<'a>, space_before: bool, space_after: bool) {
        if space_before && self.space_after {
            self.doc += self.printer.arena.space();
        }
        self.doc += doc;
        self.space_after = space_after;
        self.at_line_start = false;
    }

    pub fn enter_new_line(&mut self) {
        self.at_line_start = true;
    }

    pub fn into_doc(self) -> ArenaDoc<'a> {
        self.doc
    }
}
