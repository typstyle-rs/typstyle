use rustc_hash::FxHashMap;
use typst_syntax::{Span, SyntaxKind, SyntaxNode, ast};

use crate::ext::StrExt;

#[derive(Debug, Clone, Default)]
pub struct Attributes {
    /// Indicates whether formatting is explicitly disabled (`@typstyle off`) or always ignored.
    pub(self) is_format_disabled: bool,

    /// Indicates whether any child node contains a comment.
    pub(self) has_comment: bool,

    /// Indicates whether any descendant has a multiline string or raw.
    pub(self) has_multiline_str: bool,

    /// Indicates whether any descendant has a `MathAlignPoint`.
    pub(self) has_math_align_point: bool,

    /// Indicates whether the node text contains a linebreak.
    /// Currently, it is only used for equations.
    pub(self) is_multiline: bool,
}

/// A storage structure that manages formatting attributes for syntax nodes.
#[derive(Debug, Default)]
pub struct AttrStore {
    /// A mapping between syntax node spans and their associated attributes.
    attr_map: FxHashMap<Span, Attributes>,
}

impl AttrStore {
    /// Creates a new `AttrStore` by computing formatting-related attributes
    /// for all descendants of the given syntax node.
    pub fn new(node: &SyntaxNode) -> AttrStore {
        if node.erroneous() {
            return Default::default(); // No attributes for erroneous nodes
        }
        let mut store = AttrStore::default();
        store.compute_no_format(node);
        store.compute_multiline(node);
        store.compute_math_align_point(node);
        store
    }

    /// Checks if a given syntax node contains a comment.
    pub fn has_comment(&self, node: &SyntaxNode) -> bool {
        self.check_node_attr(node, |attr| attr.has_comment)
    }

    pub fn has_multiline_str(&self, node: &SyntaxNode) -> bool {
        self.check_node_attr(node, |attr| attr.has_multiline_str)
    }

    pub fn has_math_align_point(&self, node: &SyntaxNode) -> bool {
        self.check_node_attr(node, |attr| attr.has_math_align_point)
    }

    pub fn can_align_in_math(&self, node: &SyntaxNode) -> bool {
        self.check_node_attr(node, |attr| {
            attr.has_math_align_point && !attr.has_multiline_str
        })
    }

    /// Checks if a given syntax node or any of its descendants contains a linebreak.
    pub fn is_multiline(&self, node: &SyntaxNode) -> bool {
        self.check_node_attr(node, |attr| attr.is_multiline)
    }

    /// Checks if formatting is explicitly disabled for a given syntax node.
    pub fn is_format_disabled(&self, node: &SyntaxNode) -> bool {
        self.check_node_attr(node, |attr| attr.is_format_disabled)
    }

    fn check_node_attr(&self, node: &SyntaxNode, pred: impl FnOnce(&Attributes) -> bool) -> bool {
        self.attr_map.get(&node.span()).is_some_and(pred)
    }
}

impl AttrStore {
    fn compute_multiline(&mut self, root: &SyntaxNode) {
        self.compute_multiline_impl(root);
    }

    fn compute_multiline_impl(&mut self, node: &SyntaxNode) -> (bool, bool) {
        let mut is_multiline = false;
        let mut has_multiline_str = false;
        for child in node.children() {
            match child.kind() {
                SyntaxKind::Space => {
                    if child.text().has_linebreak() {
                        is_multiline = true;
                    }
                }
                SyntaxKind::BlockComment => {
                    is_multiline |= child.text().has_linebreak();
                }
                SyntaxKind::Str => {
                    has_multiline_str |= child.text().has_linebreak();
                }
                SyntaxKind::Raw => {
                    let raw = child.cast::<ast::Raw>().expect("raw");
                    has_multiline_str |= !raw.block() && raw.lines().nth(1).is_some();
                }
                _ => {}
            }
            let res = self.compute_multiline_impl(child);
            is_multiline |= res.0;
            has_multiline_str |= res.1;
        }
        if is_multiline {
            self.attrs_mut_of(node).is_multiline = true;
        }
        if has_multiline_str {
            self.attrs_mut_of(node).has_multiline_str = true;
        }
        (is_multiline, has_multiline_str)
    }

    fn compute_no_format(&mut self, root: &SyntaxNode) {
        self.compute_no_format_impl(root);
    }

    fn compute_no_format_impl(&mut self, node: &SyntaxNode) {
        let mut disable_next = false;
        let mut commented = false;
        for child in node.children() {
            match child.kind() {
                SyntaxKind::LineComment | SyntaxKind::BlockComment => {
                    commented = true;
                    // @typstyle off affects the whole next block
                    disable_next = child.text().contains("@typstyle off");
                }
                SyntaxKind::Space | SyntaxKind::Hash => {}
                SyntaxKind::Code | SyntaxKind::Math if disable_next => {
                    // no format nodes with @typstyle off
                    self.disable_first_nontrivial_child(child);
                    disable_next = false;
                }
                _ if disable_next => {
                    // no format nodes with @typstyle off
                    if !child.kind().is_trivia() {
                        self.attrs_mut_of(child).is_format_disabled = true;
                    }
                    disable_next = false;
                }
                _ => {
                    if !child.kind().is_trivia() {
                        self.compute_no_format_impl(child);
                    }
                }
            }
        }
        if commented {
            self.attrs_mut_of(node).has_comment = true;
        }
    }

    fn disable_first_nontrivial_child(&mut self, node: &SyntaxNode) {
        node.children()
            .find(|it| !matches!(it.kind(), SyntaxKind::Space | SyntaxKind::Hash))
            .inspect(|it| self.attrs_mut_of(it).is_format_disabled = true);
    }

    fn compute_math_align_point(&mut self, root: &SyntaxNode) {
        self.compute_math_align_point_impl(root);
    }

    fn compute_math_align_point_impl(&mut self, node: &SyntaxNode) -> bool {
        let node_kind = node.kind();
        if node_kind == SyntaxKind::MathAlignPoint {
            return true;
        }
        if node_kind.is_trivia() {
            return false;
        }
        let mut has_math_align_point = false;
        for child in node.children() {
            has_math_align_point |= self.compute_math_align_point_impl(child);
        }
        if has_math_align_point && matches!(node_kind, SyntaxKind::Math | SyntaxKind::MathDelimited)
        {
            self.attrs_mut_of(node).has_math_align_point = true;
            true
        } else {
            false
        }
    }

    fn attrs_mut_of(&mut self, node: &SyntaxNode) -> &mut Attributes {
        self.attr_map.entry(node.span()).or_default()
    }
}
