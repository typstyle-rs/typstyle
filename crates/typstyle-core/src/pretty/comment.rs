use typst_syntax::{SyntaxKind, SyntaxNode};

use super::{prelude::*, Context, PrettyPrinter};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_comment(&'a self, _ctx: Context, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        comment(&self.arena, node)
    }
}

enum CommentStyle {
    Plain,
    Bullet,
}

/// Convert either line comment or block comment.
pub fn comment<'a>(arena: &'a Arena<'a>, node: &'a SyntaxNode) -> ArenaDoc<'a> {
    if node.kind() == SyntaxKind::LineComment {
        line_comment(arena, node)
    } else if node.kind() == SyntaxKind::BlockComment {
        block_comment(arena, node)
    } else {
        unreachable!("the node should not be a comment node!")
    }
}

pub fn line_comment<'a>(arena: &'a Arena<'a>, node: &'a SyntaxNode) -> ArenaDoc<'a> {
    arena.text(node.text().as_str())
}

/// It does not add a hardline to the doc.
pub fn block_comment<'a>(arena: &'a Arena<'a>, node: &'a SyntaxNode) -> ArenaDoc<'a> {
    // Calculate the number of leading spaces except the first line.
    let line_num = node.text().lines().count();
    if line_num == 0 {
        return arena.text(node.text().as_str());
    }
    // Then the comment is multiline.
    let text = node.text().as_str();
    let style = get_comment_style(text);
    match style {
        CommentStyle::Plain => align_multiline(arena, text),
        CommentStyle::Bullet => align_multiline_simple(arena, text),
    }
}

fn get_comment_style(text: &str) -> CommentStyle {
    if text
        .lines()
        .skip(1)
        .all(|line| line.trim_start().starts_with('*'))
    {
        CommentStyle::Bullet // /*
    } else {
        CommentStyle::Plain // otherwise
    }
}

/// Get the minimum number of leading spaces in all lines except the first.
/// Returns None only when the text is a single line.
fn get_follow_leading(text: &str) -> Option<usize> {
    text.lines()
        .skip(1)
        .map(|line| line.chars().position(|c| c != ' ').unwrap_or(usize::MAX))
        .min()
}

/// For general cases. All lines need to be indented together.
fn align_multiline<'a>(arena: &'a Arena<'a>, text: &'a str) -> ArenaDoc<'a> {
    let leading = get_follow_leading(text).unwrap();
    let mut doc = arena.nil();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            doc += line;
        } else {
            doc += arena.hardline();
            if line.len() > leading {
                doc += &line[leading..]; // Remove line prefix
            } // otherwise this line is blank
        }
    }
    doc.align()
}

/// For special cases. All lines can be indented independently.
fn align_multiline_simple<'a>(arena: &'a Arena<'a>, text: &'a str) -> ArenaDoc<'a> {
    let mut doc = arena.nil();
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            doc += arena.hardline();
        }
        doc += line.trim_start();
    }
    doc.hang(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        let cmt = "/* 0
      --- 1
        -- 2
    --- 3
     -- 4 */";
        let arena = Arena::new();
        let leading = get_follow_leading(cmt).unwrap();
        assert_eq!(leading, 4);
        let doc = arena.text("lorem ipsum") + arena.space() + align_multiline(&arena, cmt);
        let result = doc.print(80).to_string();
        // println!("{result}");
        assert_eq!(
            result,
            "lorem ipsum /* 0
              --- 1
                -- 2
            --- 3
             -- 4 */"
        );
    }

    #[test]
    fn test_align2() {
        let cmt = "/* 0
      * 1
        * 2
    * 3
      */";
        let arena = Arena::new();
        let doc = arena.text("lorem ipsum") + arena.space() + align_multiline_simple(&arena, cmt);
        let result = doc.print(80).to_string();
        // println!("{result}");
        assert_eq!(
            result,
            "lorem ipsum /* 0
             * 1
             * 2
             * 3
             */"
        );
    }
}
