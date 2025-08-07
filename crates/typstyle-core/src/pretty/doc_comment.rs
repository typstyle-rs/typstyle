use itertools::Itertools;
use typst_syntax::{Source, SyntaxNode};

use crate::{
    pretty::{prelude::*, Context, PrettyPrinter},
    Config, Typstyle,
};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn format_doc_comments(
        &'a self,
        ctx: Context,
        doc_comment_nodes: Vec<&'a SyntaxNode>,
    ) -> ArenaDoc<'a> {
        let text = doc_comment_nodes
            .iter()
            .map(|&it| &it.text()[3..])
            .join("\n");
        let source = Source::detached(text);

        let config = Config {
            wrap_text: self.config.wrap_text | self.config.wrap_doc_comments,
            max_width: self.config.doc_comment_width,
            ..self.config
        };

        let Ok(formatted) = Typstyle::new(config).format_source(source).render() else {
            // Fall back to original formatting
            return self.arena.intersperse(
                doc_comment_nodes
                    .into_iter()
                    .map(|node| self.convert_comment(ctx, node)),
                self.arena.hardline(),
            );
        };
        self.arena.intersperse(
            formatted
                .lines()
                .map(|line| self.arena.text(format!("/// {line}"))),
            self.arena.hardline(),
        )
    }
}
