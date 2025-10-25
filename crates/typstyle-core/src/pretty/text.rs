use typst_syntax::{ast::*, SyntaxNode};

use super::{prelude::*, Context, PrettyPrinter};
use crate::ext::StrExt;

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_text(&'a self, text: Text<'a>) -> ArenaDoc<'a> {
        // `Text` only consists of words joined by single spaces
        self.convert_literal(text.get())
    }

    pub(super) fn convert_text_wrapped(&'a self, text: Text<'a>) -> ArenaDoc<'a> {
        wrap_text(&self.arena, text.get())
    }

    pub(super) fn convert_space(&'a self, ctx: Context, space: Space<'a>) -> ArenaDoc<'a> {
        self.convert_space_untyped(ctx, space.to_untyped())
    }

    pub(super) fn convert_space_untyped(
        &'a self,
        ctx: Context,
        node: &'a SyntaxNode,
    ) -> ArenaDoc<'a> {
        if node.text().has_linebreak() {
            self.arena.hard_line()
        } else if ctx.mode.is_markup() && !self.config.collapse_markup_spaces {
            self.arena.text(node.text().as_str())
        } else {
            self.arena.space()
        }
    }

    pub(super) fn convert_parbreak(&'a self, parbreak: Parbreak) -> ArenaDoc<'a> {
        let newline_count = parbreak.to_untyped().text().count_linebreaks();
        self.arena.hard_line().repeat(newline_count)
    }
}

/// Wraps `text` into a `ArenaDoc`, using `softline()` between words,
/// except before tokens that can be parsed as enum markers, where we use a hard space.
///
/// See: https://github.com/typst/typst/blob/8ace67d942a4b8c6b9d95b73b3a39f5d0259c7b2/crates/typst-syntax/src/lexer.rs#L479-L488
fn wrap_text<'a>(arena: &'a Arena<'a>, text: &'a str) -> ArenaDoc<'a> {
    let mut tokens = text.split_ascii_whitespace();
    // start with first token (or nil() if empty)
    let mut doc = if let Some(first) = tokens.next() {
        arena.text(first)
    } else {
        return arena.nil();
    };

    // fold over the rest
    doc = tokens.fold(doc, |doc, token| {
        let sep = if is_enum_marker(token) {
            arena.space()
        } else {
            arena.softline()
        };
        doc + sep + arena.text(token)
    });

    // preserve a trailing space as a final softline
    // special case when a link follows the text
    if text.ends_with(' ') {
        doc += arena.softline();
    }

    doc
}

/// Returns `true` if `token` is one or more ASCII digits that parse as a `usize`, followed by a dot.
/// Examples: "1.", "42.",
pub(super) fn is_enum_marker(token: &str) -> bool {
    // Check and strip trailing dot
    if let Some(stripped) = token.strip_suffix('.') {
        // Attempt to parse the digits as a `usize`
        return stripped.parse::<usize>().is_ok();
    }
    false
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn enum_marker_glues() {
        let text = "a. 1. 01. 18446744073709551615. 18446744073709551616. 18446744073709551617.";

        let arena = Arena::new();
        let doc = wrap_text(&arena, text);

        assert_snapshot!(doc.print(0).to_string(), @r"
        a. 1. 01. 18446744073709551615.
        18446744073709551616.
        18446744073709551617.
        ")
    }
}
