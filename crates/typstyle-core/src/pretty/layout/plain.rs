use typst_syntax::{ast::AstNode, SyntaxKind, SyntaxNode};

use crate::{
    ext::StrExt,
    pretty::{layout::flow::FlowStylist, prelude::*, Context, PrettyPrinter},
};

#[derive(Debug)]
enum PlainItem<'a> {
    Item(ArenaDoc<'a>),
    Comma,
    Linebreak(usize),
    LineComment(ArenaDoc<'a>),
    BlockComment(ArenaDoc<'a>),
}

/// A stylist that keeps the original structure.
pub struct PlainStylist<'a> {
    printer: &'a PrettyPrinter<'a>,
    items: Vec<PlainItem<'a>>,
    is_multiline: bool,
}

impl<'a> PlainStylist<'a> {
    pub fn new(printer: &'a PrettyPrinter<'a>) -> Self {
        Self {
            printer,
            items: Default::default(),
            is_multiline: false,
        }
    }

    #[allow(unused)]
    pub fn process<T: AstNode<'a>>(
        self,
        ctx: Context,
        node: &'a SyntaxNode,
        item_converter: impl Fn(Context, T) -> ArenaDoc<'a>,
    ) -> Self {
        self.process_iterable(ctx, node.children(), item_converter)
    }

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
                    let newline_count = child.text().count_linebreaks();
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
