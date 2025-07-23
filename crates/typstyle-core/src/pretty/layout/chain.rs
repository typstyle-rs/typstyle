use std::iter;

use itertools::Itertools;
use typst_syntax::{SyntaxKind, SyntaxNode};

use crate::{
    ext::StrExt,
    pretty::{prelude::*, util::is_comment_node, Context, PrettyPrinter},
};

/// Comment positioning strategy in chain formatting.
#[derive(Clone, Copy)]
enum CommentPosition {
    /// Regular comment that flows with the chain
    Normal,
    /// Comment that should remain on its own line
    Standalone,
    /// Comment that should be attached to the previous element
    Attached,
}

/// Intermediate representation in chain formatting.
enum ChainItem<'a> {
    Body(ArenaDoc<'a>),
    Op(ArenaDoc<'a>),
    Comment(ArenaDoc<'a>, CommentPosition),
    Linebreak,
}

/// A stylist that can format items as chains.
pub struct ChainStylist<'a> {
    printer: &'a PrettyPrinter<'a>,
    items: Vec<ChainItem<'a>>,
    /// The number of chain operators in the chain.
    chain_op_num: usize,
    /// Whether the chain contains any line or block comment.
    has_comment: bool,
}

#[derive(Default)]
pub struct ChainStyle {
    /// Do not break line if the chain consists of only one operator.
    pub no_break_single: bool,
    /// Add space before and after operators.
    pub space_around_op: bool,
}

impl<'a> ChainStylist<'a> {
    pub fn new(printer: &'a PrettyPrinter<'a>) -> Self {
        Self {
            printer,
            items: Default::default(),
            chain_op_num: 0,
            has_comment: false,
        }
    }

    /// Processes a collection of syntax nodes directly from depth-first resolution.
    ///
    /// This method takes an iterator of `SyntaxNode`s, which are then processed in reverse order.
    ///
    /// # Parameters
    ///
    /// - `nodes`: An iterator over references to `SyntaxNode`s that have been resolved.
    /// - Others: See [`Self::process`].
    pub fn process_resolved(
        self,
        ctx: Context,
        nodes: impl Iterator<Item = &'a SyntaxNode>,
        operand_pred: impl Fn(&'a SyntaxNode) -> bool,
        op_converter: impl Fn(&'a SyntaxNode) -> Option<ArenaDoc<'a>>,
        rhs_converter: impl Fn(Context, &'a SyntaxNode) -> Option<ArenaDoc<'a>>,
        fallback_converter: impl Fn(Context, &'a SyntaxNode) -> Option<ArenaDoc<'a>>,
    ) -> Self {
        let mut nodes = nodes.collect_vec();
        nodes.reverse();
        self.process(
            ctx,
            nodes,
            operand_pred,
            op_converter,
            rhs_converter,
            fallback_converter,
        )
    }

    /// Processes a vector of syntax nodes with the provided predicates and converters
    /// to create a structured representation.
    ///
    /// # Parameters
    ///
    /// - `nodes`: A vector of `SyntaxNode`s to be processed.
    /// - `operand_pred`: A predicate that checks if a node is an operand.
    /// - `op_converter`: A function that converts operators into Docs (if some).
    /// - `rhs_converter`: A function that converts right-hand side nodes into Docs (if some).
    /// - `fallback_converter`: A function that provides a fallback conversion for nodes that
    ///   do not match the primary criteria. Used for sticky args and innermost expressions.
    pub fn process(
        mut self,
        ctx: Context,
        nodes: Vec<&'a SyntaxNode>,
        operand_pred: impl Fn(&'a SyntaxNode) -> bool,
        op_converter: impl Fn(&'a SyntaxNode) -> Option<ArenaDoc<'a>>,
        rhs_converter: impl Fn(Context, &'a SyntaxNode) -> Option<ArenaDoc<'a>>,
        fallback_converter: impl Fn(Context, &'a SyntaxNode) -> Option<ArenaDoc<'a>>,
    ) -> Self {
        let mut can_attach = false;
        let mut seen_linebreak_recently = false;
        
        for node in nodes {
            if operand_pred(node) {
                self.chain_op_num += 1;
                let mut seen_op = false;
                for child in node.children() {
                    if let Some(op) = op_converter(child) {
                        seen_op = true;
                        self.items.push(ChainItem::Op(op));
                        seen_linebreak_recently = false;
                    } else if child.kind() == SyntaxKind::Space {
                        if child.text().has_linebreak() {
                            if self.items.last().is_some_and(|last| {
                                matches!(*last, ChainItem::Comment(_, _))
                            }) {
                                self.items.push(ChainItem::Linebreak);
                            }
                            can_attach = false;
                            seen_linebreak_recently = true;
                        }
                    } else if is_comment_node(child) {
                        let doc = self.printer.convert_comment(ctx, child);
                        
                        // For line comments that follow a recent linebreak, treat them as standalone
                        let is_line_comment = child.kind() == SyntaxKind::LineComment;
                        let should_be_standalone = is_line_comment && seen_linebreak_recently;
                        
                        let position = if can_attach && !should_be_standalone {
                            CommentPosition::Attached
                        } else if should_be_standalone {
                            CommentPosition::Standalone
                        } else {
                            CommentPosition::Normal
                        };
                        self.items.push(ChainItem::Comment(doc, position));
                        self.has_comment = true;
                        seen_linebreak_recently = false; // Reset after processing comment
                    } else if seen_op {
                        if let Some(rhs) = rhs_converter(ctx, child) {
                            self.items.push(ChainItem::Body(rhs));
                            can_attach = true;
                            seen_linebreak_recently = false;
                        }
                    }
                }
            } else if let Some(fallback) = fallback_converter(ctx, node) {
                // We must use this to handle args.
                if let Some(ChainItem::Body(body)) = self.items.last_mut() {
                    *body += fallback;
                } else {
                    self.items.push(ChainItem::Body(fallback));
                }
            }
        }

        self
    }

    /// Create a Doc from IR and given styles.
    pub fn print_doc(self, sty: ChainStyle) -> ArenaDoc<'a> {
        let arena = &self.printer.arena;

        let op_sep = if sty.space_around_op {
            arena.line()
        } else {
            arena.line_()
        };

        let use_simple_layout = self.chain_op_num == 1 && sty.no_break_single && !self.has_comment;

        let mut docs = vec![];
        let mut has_break = false;
        let mut leading = true;
        let mut space_after = true;
        for item in self.items {
            match item {
                ChainItem::Body(body) => {
                    if leading {
                        docs.push(body);
                    } else if let Some(last) = docs.last_mut() {
                        *last += body;
                    }
                    leading = false;
                    space_after = true;
                }
                ChainItem::Op(op) => {
                    if !(has_break && leading || use_simple_layout) {
                        docs.push(op_sep.clone());
                    }
                    has_break = false;
                    if sty.space_around_op {
                        docs.push(op + " ");
                    } else {
                        docs.push(op);
                    }
                    leading = false;
                    space_after = false;
                }
                ChainItem::Comment(cmt, position) => {
                    match position {
                        CommentPosition::Normal => {
                            if leading {
                                docs.push(cmt);
                            } else if let Some(last) = docs.last_mut() {
                                *last += if space_after {
                                    arena.space() + cmt
                                } else {
                                    cmt
                                }
                            }
                            leading = false;
                            space_after = true;
                        }
                        CommentPosition::Standalone => {
                            if leading {
                                docs.push(cmt);
                            } else {
                                // For standalone comments, put them on a new line
                                docs.push(arena.hardline());
                                docs.push(cmt);
                                leading = true;
                            }
                            space_after = true;
                        }
                        CommentPosition::Attached => {
                            if let Some(last) = docs.last_mut() {
                                *last += if space_after {
                                    arena.space() + cmt
                                } else {
                                    cmt
                                }
                            }
                        }
                    }
                }
                ChainItem::Linebreak => {
                    has_break = true;
                    leading = true;
                    docs.push(arena.hardline());
                }
            }
        }

        let first_doc = docs.remove(0);
        let follow_docs = arena.concat(docs);
        if use_simple_layout {
            first_doc + follow_docs
        } else {
            first_doc + self.printer.indent(follow_docs)
        }
        .group()
    }
}

/// Iterates over nodes in a syntax tree in a depth-first manner.
///
/// This function takes a starting node and an `accepter` function,
/// which determines the next node to visit. It returns an iterator that yields
/// nodes in a depth-first order based on the logic defined in the `accepter`.
///
/// # Parameters
///
/// - `node`: The starting node from which to begin the iteration.
/// - `accepter`: If it returns `Some`, the iterator will visit this next node;
///   if it returns `None`, the iteration will stop for that path.
pub fn iterate_deep_nodes<'a>(
    node: &'a SyntaxNode,
    accepter: impl Fn(&SyntaxNode) -> Option<&SyntaxNode> + 'a,
) -> impl Iterator<Item = &'a SyntaxNode> {
    let mut current = Some(node);
    iter::from_fn(move || {
        let ret = current;
        if let Some(ret) = ret {
            current = accepter(ret);
            Some(ret)
        } else {
            None
        }
    })
}
