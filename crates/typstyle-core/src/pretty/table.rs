use typst_syntax::{SyntaxKind, SyntaxNode, ast::*};

use super::{Context, prelude::*, util::func_name};
use crate::{
    PrettyPrinter,
    ext::StrExt,
    pretty::{Mode, layout::table::TableCollector},
};

impl<'a> PrettyPrinter<'a> {
    pub(super) fn try_convert_table(
        &'a self,
        ctx: Context,
        table: FuncCall<'a>,
        paren_nodes: &'a [SyntaxNode],
    ) -> Option<ArenaDoc<'a>> {
        // NOTE: args are not empty here
        let columns = if is_table(table) && is_table_formattable(table, paren_nodes) {
            get_table_columns(table)
        } else {
            None
        }?;
        Some(self.convert_table(ctx, paren_nodes, columns))
    }

    /// Handle parenthesized args of a table.
    pub(super) fn convert_table(
        &'a self,
        ctx: Context,
        paren_nodes: &'a [SyntaxNode],
        columns: usize,
    ) -> ArenaDoc<'a> {
        let ctx = ctx.with_mode(Mode::CodeCont);

        // Rules:
        // - named/spread args, header/footer: occupy a line.
        // - reflow cells if no special cells (cell, hline, vline, )
        // - hard break at linebreaks with at least 1 empty lines
        let can_reflow_cells = paren_nodes
            .iter()
            .any(|it| it.cast().is_some_and(is_special_cell));
        let mut collector =
            TableCollector::new(&self.arena, if can_reflow_cells { 0 } else { columns });

        for node in paren_nodes.iter() {
            if let Some(arg) = node.cast::<Arg>() {
                match arg {
                    Arg::Pos(Expr::FuncCall(func_call)) if is_header_footer(func_call) => {
                        // This func_call does not pass the escape-hatch check in `convert_expr`.
                        let doc = if let Some(res) = self.check_disabled(func_call.to_untyped()) {
                            res
                        } else {
                            self.convert_expr(ctx, func_call.callee())
                                + self.convert_args(ctx, func_call.args(), |nodes| {
                                    self.convert_table(ctx, nodes, columns)
                                })
                        };
                        collector.push_row(doc);
                    }
                    Arg::Pos(expr) => {
                        collector.push_cell(self.convert_expr(ctx, expr));
                    }
                    Arg::Named(named) => {
                        collector.push_row(self.convert_named(ctx, named));
                    }
                    Arg::Spread(spread) => {
                        // NOTE: when spread exists, regarding it as a cell will not affect layout.
                        collector.push_cell(self.convert_spread(ctx, spread));
                    }
                }
            } else if node.kind() == SyntaxKind::Space {
                collector.push_newline(node.text().count_linebreaks());
            } else if node.kind() == SyntaxKind::LineComment {
                collector.push_comment(self.convert_comment(ctx, node));
            };
        }
        let doc = collector.collect();
        self.block_indent(doc).group().parens()
    }
}

pub fn is_table(func_call: FuncCall) -> bool {
    matches!(func_name(func_call), Some("table") | Some("grid"))
}

fn is_table_formattable(func_call: FuncCall, paren_nodes: &[SyntaxNode]) -> bool {
    // 1. no block comments
    if func_call
        .args()
        .to_untyped()
        .children()
        .any(|it| matches!(it.kind(), SyntaxKind::BlockComment))
    {
        return false;
    }
    // 2. has at least one pos arg
    paren_nodes
        .iter()
        .any(|it| matches!(it.cast::<Arg>(), Some(Arg::Pos(_))))
}

fn get_table_columns(func_call: FuncCall) -> Option<usize> {
    use crate::liteval::{Liteval, Value};

    let Some(columns_expr) = func_call.args().items().find_map(|node| {
        if let Arg::Named(named) = node
            && named.name().as_str() == "columns"
        {
            return Some(named.expr());
        }
        None
    }) else {
        return if (func_call.args().items()).any(|arg| matches!(arg, Arg::Spread(_))) {
            None // the columns may be provided in spread args.
        } else {
            Some(1) // if not `columns` is provided, regard as 1.
        };
    };
    match columns_expr.liteval() {
        Ok(Value::Auto) => Some(1),
        Ok(Value::Int(i)) => Some(i as usize),
        Ok(Value::Array(a)) => Some(a),
        _ => None,
    }
}

fn is_header_footer(func_call: FuncCall) -> bool {
    const HEADER_FOOTER: &[&str] = &["header", "footer"];

    func_name(func_call).is_some_and(|name| HEADER_FOOTER.contains(&name))
}

fn is_special_cell(arg: Arg) -> bool {
    const BLACK_LIST: &[&str] = &["cell", "vline", "hline"];

    match arg {
        Arg::Pos(Expr::FuncCall(func_call)) => {
            func_name(func_call).is_some_and(|name| BLACK_LIST.contains(&name))
        }
        Arg::Spread(_) => true,
        _ => false,
    }
}
