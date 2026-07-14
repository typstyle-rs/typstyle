use typst_syntax::{SyntaxKind, SyntaxNode};

use super::{Context, PrettyPrinter, prelude::*};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_comment(&'a self, _ctx: Context, node: &'a SyntaxNode) -> ArenaDoc<'a> {
        comment(&self.arena, node)
    }
}

/// Style of block comment lines.
/// ```text
/// /*
///  A plain block comment line
/// */
///
/// /*
/// * A bullet block comment line
/// */
/// ```
enum BlockCommentLineStyle {
    /// Block comment lines with optional leading whitespace.
    Plain,
    /// Block comment lines contain leading asterisks.
    Bullet,
}

/// Convert either line comment or block comment. Line comments are converted as line suffixes.
pub fn comment<'a>(arena: &'a Arena<'a>, node: &'a SyntaxNode) -> ArenaDoc<'a> {
    match node.kind() {
        SyntaxKind::LineComment => line_comment(arena, node).as_line_suffix(),
        SyntaxKind::BlockComment => block_comment(arena, node),
        _ => unreachable!("This node should not be a comment node!")
    }
}

/// Format line comment as regular text.
pub fn line_comment<'a>(arena: &'a Arena<'a>, node: &'a SyntaxNode) -> ArenaDoc<'a> {
    arena.text(node.leaf_text().as_str())
}

/// Format block comments. They do not add a hardline to the doc.
pub fn block_comment<'a>(arena: &'a Arena<'a>, node: &'a SyntaxNode) -> ArenaDoc<'a> {
    let text = node.leaf_text().as_str();
    let style = get_block_comment_line_style(text);
    match style {
        BlockCommentLineStyle::Plain => align_multiline_together(arena, text),
        BlockCommentLineStyle::Bullet => align_multiline_independent(arena, text),
    }
}

/// Gets the line style of block comment lines from the given text.
/// Single-line block comments are treated as [`BlockCommentLineStyle::Bullet`].
fn get_block_comment_line_style(text: &str) -> BlockCommentLineStyle {
    if text
        .lines()
        .skip(1)
        .all(|line| line.trim_start().starts_with('*'))
    {
        BlockCommentLineStyle::Bullet
    } else {
        BlockCommentLineStyle::Plain
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

/// Indents all lines of the block comment to the same level. Used for [`BlockCommentLineStyle::Plain`].
fn align_multiline_together<'a>(arena: &'a Arena<'a>, text: &'a str) -> ArenaDoc<'a> {
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

/// Indents all lines of the block comment independently. Used for [`BlockCommentLineStyle::Bullet`].
fn align_multiline_independent<'a>(arena: &'a Arena<'a>, text: &'a str) -> ArenaDoc<'a> {
    let mut doc = arena.nil();
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            doc += arena.hardline();
        }
        doc += line.trim_start();
    }
    doc.nest(1).align()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_plain_block_comment() {
        let cmt = "/* 0
      --- 1
        -- 2
    --- 3
     -- 4 */";
        let arena = Arena::new();
        let leading = get_follow_leading(cmt).unwrap();
        assert_eq!(leading, 4);
        
        let doc = arena.text("lorem ipsum") + arena.space() + align_multiline_together(&arena, cmt);
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
    fn test_align_bullet_block_comment() {
        let cmt = "/* 0
      * 1
        * 2
    * 3
      */";
        let arena = Arena::new();
        let doc = arena.text("lorem ipsum") + arena.space() + align_multiline_independent(&arena, cmt);
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
