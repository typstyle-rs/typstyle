//! Lays out a sequence of items with configurable spacing and interleaved comments.
//!
//! Used by the converters for control-flow and other flow-like constructs.

use crate::pretty::{PrettyPrinter, prelude::*};

/// An item in the flow. A space is added only when the item before and the item after both allow it.
pub struct FlowItem<'a>(pub Option<FlowItemRepr<'a>>);

/// An item with its document and spacing permissions.
pub struct FlowItemRepr<'a> {
    /// The document of the item.
    pub doc: ArenaDoc<'a>,
    /// Whether a space can be added before the doc.
    pub space_before: bool,
    /// Whether a space can be added after the doc.
    pub space_after: bool,
}

impl<'a> FlowItem<'a> {
    /// Creates an item with explicit spacing permissions.
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

    /// Creates an empty item that is skipped when printing.
    pub const fn none() -> Self {
        Self(None)
    }
}

/// Lays out a sequence of items into one document, inserting spaces where both
/// neighboring items allow them.
pub struct FlowStylist<'a> {
    /// The printer that provides the arena.
    printer: &'a PrettyPrinter<'a>,
    /// The document accumulated so far.
    doc: ArenaDoc<'a>,
    /// Whether the item pushed last allows a space after it.
    space_after: bool,
    /// Whether nothing has been pushed since the last explicit line break.
    at_line_start: bool,
}

impl<'a> FlowStylist<'a> {
    /// Creates a new stylist with an empty flow.
    pub fn new(printer: &'a PrettyPrinter<'a>) -> Self {
        Self {
            doc: printer.arena.nil(),
            printer,
            space_after: false,
            at_line_start: true,
        }
    }

    /// Pushes a comment into the flow.
    ///
    /// Block comments allow spaces on both sides; line comments allow a space before
    /// them (unless at line start) but none after.
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

    /// Pushes a document, inserting a space before it if both the previous item and
    /// this one allow a space.
    pub fn push_doc(&mut self, doc: ArenaDoc<'a>, space_before: bool, space_after: bool) {
        if space_before && self.space_after {
            self.doc += self.printer.arena.weak_space();
        }
        self.doc += doc;
        self.space_after = space_after;
        self.at_line_start = false;
    }

    /// Marks the current position as a line start, so a following line comment is
    /// not preceded by a space.
    pub fn enter_new_line(&mut self) {
        self.at_line_start = true;
    }

    /// Returns the accumulated document.
    pub fn into_doc(self) -> ArenaDoc<'a> {
        self.doc
    }
}
