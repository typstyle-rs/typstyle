//! Lays out a plain sequence of items without delimiters.
//!
//! Source line breaks and comments are preserved as separate items.

use typst_syntax::{SyntaxKind, SyntaxNode, ast::AstNode};

use crate::{
    ext::StrExt,
    pretty::{Context, PrettyPrinter, layout::flow::FlowStylist, prelude::*},
};

/// An item collected by [`PlainStylist`].
#[derive(Debug)]
enum PlainItem<'a> {
    /// A converted child node.
    Item(ArenaDoc<'a>),
    /// A comma between items.
    Comma,
    /// Line breaks preserved from the source, capped by the blank-lines upper bound.
    Linebreak(usize),
    /// A line comment, which forces the output onto multiple lines.
    LineComment(ArenaDoc<'a>),
    /// A block comment.
    BlockComment(ArenaDoc<'a>),
}

/// A stylist that keeps the original structure.
pub struct PlainStylist<'a> {
    /// The printer that provides the arena and comment conversion.
    printer: &'a PrettyPrinter<'a>,
    /// The collected items in source order.
    items: Vec<PlainItem<'a>>,
    /// Whether any source line break or line comment was seen.
    is_multiline: bool,
}

impl<'a> PlainStylist<'a> {
    /// Creates a new stylist with no items.
    pub fn new(printer: &'a PrettyPrinter<'a>) -> Self {
        Self {
            printer,
            items: Default::default(),
            is_multiline: false,
        }
    }

    #[allow(unused)]
    /// Processes the children of `node` as items. See [`Self::process_iterable`].
    pub fn process<T: AstNode<'a>>(
        self,
        ctx: Context,
        node: &'a SyntaxNode,
        item_converter: impl Fn(Context, T) -> ArenaDoc<'a>,
    ) -> Self {
        self.process_iterable(ctx, node.children(), item_converter)
    }

    /// Processes an iterable of nodes, converting those that can be cast to `T`.
    ///
    /// Commas are kept as separators; up to [`crate::Config::blank_lines_upper_bound`]
    /// blank lines are preserved between items; comments are collected as items.
    pub fn process_iterable<T: AstNode<'a>>(
        mut self,
        ctx: Context,
        iterable: impl Iterator<Item = &'a SyntaxNode>,
        item_converter: impl Fn(Context, T) -> ArenaDoc<'a>,
    ) -> Self {
        let nl = self.printer.config.blank_lines_upper_bound;
        for child in iterable {
            self.items.push(match child.kind() {
                SyntaxKind::Comma => PlainItem::Comma,
                SyntaxKind::Space => {
                    let newline_count = child.leaf_text().count_linebreaks();
                    if newline_count > 0 {
                        self.is_multiline = true;
                        if !self.items.is_empty() {
                            PlainItem::Linebreak(newline_count.min(nl + 1))
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                SyntaxKind::LineComment => {
                    self.is_multiline = true;
                    PlainItem::LineComment(self.printer.convert_comment(ctx, child))
                }
                SyntaxKind::BlockComment => {
                    PlainItem::BlockComment(self.printer.convert_comment(ctx, child))
                }
                _ => {
                    if let Some(item) = child.cast() {
                        PlainItem::Item(item_converter(ctx, item))
                    } else {
                        continue;
                    }
                }
            });
        }

        // Remove trailing linebreaks.
        while let Some(PlainItem::Linebreak(_)) = self.items.last() {
            self.items.pop();
        }

        self
    }

    /// Builds the document by feeding the collected items through a [`FlowStylist`].
    ///
    /// If the source was multiline, the result is enclosed in hard line breaks.
    pub fn print_doc(self) -> ArenaDoc<'a> {
        let arena = &self.printer.arena;

        let mut flow = FlowStylist::new(self.printer);
        for item in self.items {
            match item {
                PlainItem::Item(body) => flow.push_doc(body, true, true),
                PlainItem::Comma => flow.push_doc(arena.text(","), false, true),
                PlainItem::Linebreak(n) => flow.push_doc(arena.hard_line().repeat(n), false, false),
                PlainItem::LineComment(cmt) => flow.push_doc(cmt, true, false),
                PlainItem::BlockComment(cmt) => flow.push_doc(cmt, true, true),
            }
        }

        let doc = flow.into_doc();
        if self.is_multiline {
            doc.enclose(arena.hard_line(), arena.hard_line())
        } else {
            doc
        }
    }
}
