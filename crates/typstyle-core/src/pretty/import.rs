use std::collections::HashSet;

use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{
    Context, PrettyPrinter,
    layout::{
        flow::FlowItem,
        list::{ListStyle, ListStylist},
    },
    prelude::*,
    util::is_comment_node,
};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn convert_import(&'a self, ctx: Context, import: ModuleImport<'a>) -> ArenaDoc<'a> {
        // ImportItems are optional and may be wrapped in parentheses.
        let nodes = import.to_untyped().children().as_slice();

        // Find the index where import items start (either at a left parenthesis or directly as ImportItems).
        let divider_index = nodes
            .iter()
            .position(|node| {
                node.kind() == SyntaxKind::LeftParen || node.kind() == SyntaxKind::ImportItems
            })
            .unwrap_or(nodes.len());

        // Split nodes into the prefix and import items parts.
        let import_items_part = &nodes[divider_index..];
        let prefix_part =
            if divider_index > 0 && nodes[divider_index - 1].kind() == SyntaxKind::Space {
                // Remove the trailing space from the prefix.
                &nodes[..divider_index - 1]
            } else {
                &nodes[..divider_index]
            };

        // Convert the prefix section.
        let prefix_doc = self.convert_flow_like_iter(ctx, prefix_part.iter(), |ctx, child, _| {
            match child.kind() {
                SyntaxKind::Colon => FlowItem::tight_spaced(self.arena.text(":")),
                SyntaxKind::Star => FlowItem::spaced(self.arena.text("*")), // wildcard import
                _ => {
                    if let Some(ident) = child.cast() {
                        // new_name
                        FlowItem::spaced(self.convert_ident(ident))
                    } else if let Some(expr) = child.cast() {
                        // source
                        FlowItem::spaced(self.convert_expr(ctx, expr))
                    } else {
                        FlowItem::none()
                    }
                }
            }
        });

        // If there are no import items, return the prefix.
        if import_items_part.is_empty() {
            return prefix_doc;
        }

        // Flatten and collect the import item nodes.
        let mut import_items_nodes = vec![];
        for node in import_items_part.iter() {
            if let Some(items) = node.cast::<ImportItems>() {
                import_items_nodes.extend(items.to_untyped().children());
            } else {
                import_items_nodes.push(node);
            }
        }
        if import_items_nodes.is_empty() {
            return prefix_doc;
        }

        let import_items_doc = self.convert_import_items(ctx, import_items_nodes);
        prefix_doc + self.arena.space() + import_items_doc
    }

    fn convert_import_items(
        &'a self,
        ctx: Context,
        mut import_items_nodes: Vec<&'a SyntaxNode>,
    ) -> ArenaDoc<'a> {
        // Sort import items if the configuration allows it.
        // The sorting is only applied if all nodes are not comments and if there are no duplicate names.
        if self.config.reorder_import_items
            && import_items_nodes.iter().all(|node| !is_comment_node(node))
            && check_import_name_duplication(&import_items_nodes)
        {
            // Sort import items by their text representation.
            import_items_nodes.sort_by_key(|&node| node.clone().into_text());
        }
        // Note that `ImportItem` does not implement `AstNode`.
        ListStylist::new(self)
            .process_iterable_impl(
                ctx,
                import_items_nodes.into_iter(),
                |ctx, child| match child.kind() {
                    SyntaxKind::RenamedImportItem => child
                        .cast()
                        .map(|item| self.convert_import_item_renamed(ctx, item)),
                    SyntaxKind::ImportItemPath => child
                        .cast()
                        .map(|item| self.convert_import_item_path(ctx, item)),
                    _ => Option::None,
                },
            )
            .print_doc(ListStyle {
                omit_delim_flat: true,
                omit_delim_empty: true,
                soft_break: true,
                ..Default::default()
            })
    }

    fn convert_import_item_path(
        &'a self,
        ctx: Context,
        import_item_path: ImportItemPath<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, import_item_path.to_untyped(), |_ctx, child, _| {
            if child.kind() == SyntaxKind::Dot {
                FlowItem::tight(self.arena.text("."))
            } else if let Some(ident) = child.cast() {
                FlowItem::tight(self.convert_ident(ident))
            } else {
                FlowItem::none()
            }
        })
    }

    fn convert_import_item_renamed(
        &'a self,
        ctx: Context,
        import_item_renamed: RenamedImportItem<'a>,
    ) -> ArenaDoc<'a> {
        self.convert_flow_like(ctx, import_item_renamed.to_untyped(), |ctx, child, _| {
            if let Some(path) = child.cast() {
                FlowItem::spaced(self.convert_import_item_path(ctx, path))
            } else if let Some(ident) = child.cast() {
                FlowItem::spaced(self.convert_ident(ident))
            } else {
                FlowItem::none()
            }
        })
    }
}

/// Check for duplicate import names in the given import items nodes.
/// Returns `true` if no duplicates are found, `false` otherwise.
fn check_import_name_duplication(import_items_nodes: &[&SyntaxNode]) -> bool {
    let mut seen = HashSet::new();
    for node in import_items_nodes.iter() {
        let name = match node.kind() {
            SyntaxKind::ImportItemPath => node.cast::<ImportItemPath>().unwrap().name().as_str(),
            SyntaxKind::RenamedImportItem => node
                .cast::<RenamedImportItem>()
                .unwrap()
                .new_name()
                .as_str(),
            _ => continue,
        };
        if !seen.insert(name) {
            return false; // Duplicate found
        }
    }
    true // No duplicates found
}
