use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use crate::{
    ext::StrExt,
    pretty::{Context, Mode, PrettyPrinter, prelude::*, style::FoldStyle},
};

pub struct ListStylist<'a> {
    printer: &'a PrettyPrinter<'a>,

    can_attach: bool,
    free_comments: Vec<ArenaDoc<'a>>,
    peek_hash: bool,
    items: Vec<Item<'a>>,
    real_item_count: usize,
    has_comment: bool,
    has_line_comment: bool,

    fold_style: FoldStyle,
    disallow_front_comment: bool,
    disallow_comment_detach: bool,
    /// Some: max_consecutive_lines; None: ignore
    keep_linebreak: Option<usize>,
}

enum Item<'a> {
    /// Detached comments that can be put on a line.
    Comment(ArenaDoc<'a>),
    /// List item with attached comments.
    Commented {
        /// The list item.
        body: ArenaDoc<'a>,
        /// Attached comments. Leading space included.
        after: Option<ArenaDoc<'a>>,
    },
    /// Linebreaks
    Linebreak(usize),
}

pub struct ListStyle {
    /// The separator between items.
    pub separator: &'static str,
    /// The delimiter of the list.
    pub delim: (&'static str, &'static str),
    /// Whether can add linebreaks inside the delimiters.
    pub tight_delim: bool,
    /// Whether to add an addition space inside the delimiters if the list is flat.
    pub add_delim_space: bool,
    /// Whether a trailing separator is need if the list contains only one item.
    pub add_trailing_sep_single: bool,
    /// Whether a trailing separator is always needed.
    pub add_trailing_sep_always: bool,
    /// Whether can omit the delimiter if the list contains only one item.
    pub omit_delim_single: bool,
    /// Whether can omit the delimiter if the list is flat.
    pub omit_delim_flat: bool,
    /// Whether can omit the delimiter if the list is empty.
    pub omit_delim_empty: bool,
    /// Whether not to indent the items.
    pub no_indent: bool,
    /// Whether use soft linebreaks to wrap items.
    pub soft_break: bool,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            separator: ",",
            delim: ("(", ")"),
            tight_delim: false,
            add_delim_space: false,
            add_trailing_sep_single: false,
            add_trailing_sep_always: false,
            omit_delim_single: false,
            omit_delim_flat: false,
            omit_delim_empty: false,
            no_indent: false,
            soft_break: false,
        }
    }
}

impl<'a> ListStylist<'a> {
    pub fn new(printer: &'a PrettyPrinter<'a>) -> Self {
        Self {
            printer,

            can_attach: false,
            free_comments: Default::default(),
            peek_hash: false,
            items: Default::default(),
            real_item_count: 0,
            has_comment: false,
            has_line_comment: false,

            fold_style: FoldStyle::Fit,
            disallow_front_comment: false,
            disallow_comment_detach: false,
            keep_linebreak: None,
        }
    }

    pub fn keep_linebreak(mut self, count: usize) -> Self {
        self.keep_linebreak = Some(count);
        self
    }

    pub fn disallow_front_comment(mut self) -> Self {
        self.disallow_front_comment = true;
        self
    }

    pub fn with_fold_style(mut self, fold_style: FoldStyle) -> Self {
        self.fold_style = fold_style;
        if fold_style == FoldStyle::Always {
            self.disallow_comment_detach = true;
        }
        self
    }

    /// Force to fold if the predicate is true. Has no effect the list contains any comment.
    pub fn always_fold_if(mut self, pred: impl FnOnce() -> bool) -> Self {
        if !self.has_comment && pred() {
            self.fold_style = FoldStyle::Always;
        }
        self
    }
}

impl<'a> ListStylist<'a> {
    /// Process a list of `AstNode`'s.
    pub fn process_list<T: AstNode<'a>>(
        self,
        ctx: Context,
        list_node: &'a SyntaxNode,
        item_converter: impl Fn(Context, T) -> ArenaDoc<'a>,
    ) -> Self {
        self.process_list_impl(ctx, list_node, |ctx, node| {
            node.cast().map(|node| item_converter(ctx, node))
        })
    }

    /// Process a list of any nodes. Only use this when the node does not implement `AstNode`.
    pub fn process_list_impl(
        self,
        ctx: Context,
        list_node: &'a SyntaxNode,
        item_checker: impl FnMut(Context, &'a SyntaxNode) -> Option<ArenaDoc<'a>>,
    ) -> Self {
        self.process_iterable_impl(ctx, list_node.children(), item_checker)
    }

    pub fn process_iterable<T: AstNode<'a>>(
        self,
        ctx: Context,
        iterable: impl Iterator<Item = &'a SyntaxNode>,
        item_converter: impl Fn(Context, T) -> ArenaDoc<'a>,
    ) -> Self {
        self.process_iterable_impl(ctx, iterable, |ctx, node| {
            node.cast().map(|node| item_converter(ctx, node))
        })
    }

    /// Process an iterable of nodes.
    pub fn process_iterable_impl(
        mut self,
        ctx: Context,
        iterable: impl Iterator<Item = &'a SyntaxNode>,
        mut item_checker: impl FnMut(Context, &'a SyntaxNode) -> Option<ArenaDoc<'a>>,
    ) -> Self {
        // Each item can be attached with comments at the front and back.
        // Can break line after front attachments.
        // If the back attachment appears before the comma, the comma is move to its front if multiline.

        for node in iterable {
            let ctx = ctx.with_mode_if(Mode::Code, self.peek_hash);
            if let Some(item_body) = item_checker(ctx, node) {
                self.add_item(item_body);
                self.peek_hash = false;
            } else {
                self.peek_hash = false;
                self.process_trivia(ctx, node);
            }
        }

        self.process_windup();

        self
    }

    fn add_item(&mut self, item_body: ArenaDoc<'a>) {
        let arena = &self.printer.arena;

        self.real_item_count += 1;
        let before = if self.disallow_front_comment {
            self.detach_comments();
            arena.nil()
        } else if self.free_comments.is_empty() {
            arena.nil()
        } else {
            // TODO - this may work with FoldStyle::Always
            let sep = if self.disallow_comment_detach {
                arena.space()
            } else {
                arena.line()
            };
            let doc = arena.intersperse(self.free_comments.drain(..), sep.clone()) + sep;
            if self.disallow_comment_detach {
                doc
            } else {
                doc.group()
            }
        };
        let hash = if self.peek_hash {
            arena.text("#")
        } else {
            arena.nil()
        };
        self.items.push(Item::Commented {
            body: (before + hash + item_body),
            after: None,
        });
        self.can_attach = true;
    }

    fn process_trivia(&mut self, ctx: Context, node: &'a SyntaxNode) {
        match node.kind() {
            SyntaxKind::LineComment | SyntaxKind::BlockComment => {
                self.has_comment = true;
                // Line comment cannot appear in single line block
                if node.kind() == SyntaxKind::LineComment {
                    self.has_line_comment = true;
                    self.fold_style = FoldStyle::Never;
                }
                self.free_comments
                    .push(self.printer.convert_comment(ctx, node));
            }
            SyntaxKind::Comma => {
                self.try_attach_comments();
            }
            SyntaxKind::Space => {
                let newline_cnt = node.text().count_linebreaks();
                if newline_cnt > 0 {
                    self.attach_or_detach_comments();
                    self.can_attach = false;
                    if let Some(nl) = self.keep_linebreak
                        && newline_cnt >= 2
                        && !self.items.is_empty()
                    {
                        self.items.push(Item::Linebreak((newline_cnt - 1).min(nl)));
                    }
                }
            }
            SyntaxKind::Hash => {
                self.peek_hash = true;
            }
            _ => {}
        }
    }

    /// Process remaining free comments and trailing lines.
    fn process_windup(&mut self) {
        self.attach_or_detach_comments();
        while let Some(Item::Linebreak(_)) = self.items.last() {
            self.items.pop();
        }
    }

    /// Try attaching free comments. If it fails, detach them.
    fn attach_or_detach_comments(&mut self) {
        if !self.try_attach_comments() {
            self.detach_comments();
        }
    }

    /// Attack free comments to the last item if possible.
    fn try_attach_comments(&mut self) -> bool {
        if self.can_attach && !self.free_comments.is_empty() {
            let arena = &self.printer.arena;
            if let Some(Item::Commented { after, .. }) = self.items.last_mut() {
                let added =
                    arena.space() + arena.intersperse(self.free_comments.drain(..), arena.space());
                match after {
                    Some(cmt) => *cmt += added,
                    Option::None => *after = Some(added),
                }
                return true;
            }
        }
        false
    }

    /// Make all free comments detached.
    fn detach_comments(&mut self) {
        self.items
            .extend(self.free_comments.drain(..).map(Item::Comment));
    }
}

impl<'a> ListStylist<'a> {
    /// Create Doc from items in self.
    ///
    /// For attached comments:
    /// - break: `xxx, /* yyy */`, `xxx,`
    /// - flat: `xxx /* yyy */, `, `xxx, `
    pub fn print_doc(self, sty: ListStyle) -> ArenaDoc<'a> {
        let arena = &self.printer.arena;

        let delim = sty.delim;
        if self.items.is_empty() {
            return if sty.omit_delim_empty {
                arena.nil()
            } else {
                arena.text(delim.0) + delim.1
            };
        }

        let is_single = self.real_item_count == 1;
        let sep = arena.text(sty.separator);
        let fold_style = if self.has_line_comment {
            FoldStyle::Never
        } else {
            self.fold_style
        };
        let item_count = self.items.len();

        let enclose_fitted = |inner: ArenaDoc<'a>| {
            if is_single && sty.omit_delim_single {
                inner.group()
            } else if sty.omit_delim_flat {
                inner
                    .enclose(
                        arena.text(delim.0).when_group_break(),
                        arena.text(delim.1).when_group_break(),
                    )
                    .group()
            } else if sty.add_delim_space {
                inner
                    .enclose(
                        arena.text(delim.0) + arena.space().when_group_flat(),
                        arena.space().when_group_flat() + arena.text(delim.1),
                    )
                    .group()
            } else {
                inner.group().enclose(delim.0, delim.1)
            }
        };

        let mut seen_real_items = 0;
        match fold_style {
            FoldStyle::Never => {
                let mut inner = if sty.tight_delim {
                    arena.nil()
                } else {
                    arena.hardline()
                };
                for (i, item) in self.items.into_iter().enumerate() {
                    let is_last = i + 1 == item_count;
                    match item {
                        Item::Comment(cmt) => inner += cmt + arena.hardline(),
                        Item::Commented { body, after } => {
                            inner += body + sep.clone() + after;
                            if !sty.tight_delim || !is_last {
                                inner += arena.hardline();
                            }
                        }
                        Item::Linebreak(n) => inner += arena.hardline().repeat(n),
                    }
                }
                if !sty.no_indent {
                    inner = self.printer.indent(inner);
                }
                inner.enclose(delim.0, delim.1)
            }
            FoldStyle::Always => {
                // TODO - this may implies `tight_delim`
                let mut inner = arena.nil();
                for (i, item) in self.items.into_iter().enumerate() {
                    let is_last = i + 1 == item_count;
                    match item {
                        Item::Comment(cmt) => {
                            inner += if is_last && sty.tight_delim {
                                cmt
                            } else {
                                cmt + arena.space()
                            }
                        }
                        Item::Commented { body, after } => {
                            seen_real_items += 1;
                            let is_last_real = seen_real_items == self.real_item_count;
                            inner += body + after;
                            if !is_last_real {
                                inner += sep.clone() + arena.space();
                            } else if sty.add_trailing_sep_always
                                || is_single && sty.add_trailing_sep_single
                            {
                                // trailing comma for one-size array
                                inner += sep.clone();
                            }
                        }
                        Item::Linebreak(_) => (),
                    }
                }
                inner = inner.group();
                if is_single && sty.omit_delim_single || sty.omit_delim_flat {
                    inner
                } else if sty.add_delim_space {
                    inner
                        .enclose(arena.space(), arena.space())
                        .enclose(delim.0, delim.1)
                } else {
                    inner.enclose(delim.0, delim.1)
                }
            }
            FoldStyle::Compact
                if !self.has_comment
                    && !self.items.iter().any(|it| matches!(it, Item::Linebreak(_))) =>
            {
                // no line comment or linebreak here
                let mut docs = vec![];
                for item in self.items {
                    match item {
                        Item::Comment(_) => {}
                        Item::Commented { body, .. } => {
                            docs.push(body);
                        }
                        Item::Linebreak(_) => {}
                    }
                }
                let last = docs.pop().unwrap();
                let inner = if docs.is_empty() {
                    // only one item
                    let compact = if sty.add_trailing_sep_single {
                        last.clone() + sep.clone()
                    } else {
                        last.clone()
                    };
                    let expanded = self.printer.block_indent(last + sep.clone());

                    compact.partial_union(expanded)
                } else {
                    let compact = arena.concat(
                        docs.iter()
                            .map(|doc| doc.clone().flatten() + sep.clone() + arena.space()),
                    ) + last.clone();

                    let expanded = self.printer.block_indent(
                        arena.concat(
                            docs.iter()
                                .map(|doc| doc.clone() + sep.clone() + arena.line()),
                        ) + last
                            + sep.clone(),
                    );

                    compact.partial_union(expanded)
                };
                enclose_fitted(inner)
            }
            FoldStyle::Fit | FoldStyle::Compact => {
                let ln = if sty.soft_break && !self.has_line_comment {
                    arena.softline()
                } else {
                    arena.line()
                };
                let ln_last = arena.line_();

                let mut inner = if sty.tight_delim {
                    arena.nil()
                } else {
                    arena.line_()
                };
                for (i, item) in self.items.into_iter().enumerate() {
                    let is_last = i + 1 == item_count;
                    match item {
                        Item::Comment(cmt) => {
                            inner += if is_last && sty.tight_delim {
                                cmt
                            } else {
                                cmt + arena.hardline()
                            }
                        }
                        Item::Commented { body, after } => {
                            seen_real_items += 1;
                            let is_last_real = seen_real_items == self.real_item_count;
                            let follow = if let Some(after) = after {
                                let follow_break = sep.clone() + after.clone();
                                let follow_flat = if !is_last_real
                                    || sty.add_trailing_sep_always
                                    || is_single && sty.add_trailing_sep_single
                                {
                                    after + sep.clone()
                                } else {
                                    after
                                };
                                follow_break.flat_alt(follow_flat)
                            } else {
                                #[allow(clippy::let_and_return)]
                                let follow = if is_last_real && sty.tight_delim {
                                    arena.nil()
                                } else if !is_last_real
                                    || sty.add_trailing_sep_always
                                    || is_single && sty.add_trailing_sep_single
                                {
                                    sep.clone()
                                } else {
                                    sep.clone().when_group_break()
                                };
                                follow
                            };
                            let ln = if !is_last_real {
                                ln.clone()
                            } else if sty.tight_delim {
                                arena.nil()
                            } else {
                                ln_last.clone()
                            };
                            inner += body + follow + ln;
                        }
                        Item::Linebreak(n) => inner += arena.line_().repeat(n),
                    }
                }
                if !sty.no_indent {
                    inner = self.printer.indent(inner);
                }
                enclose_fitted(inner)
            }
        }
    }
}
