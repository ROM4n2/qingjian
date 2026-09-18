//! 框：flex 排子节点；带 `table` 时排成表格——`repeat` 出来的每份组件是一行，它的子节点依次是各列。
//!
//! 表格各列取各行最宽，最后补一列 `1fr` 吃掉剩余宽度，这样 `span: row` 的节点（高亮条）能横跨整个表格宽；
//! 列间距由各格的外边距给（grid 的 gap 会让补的那一列也多出一个间距）。

use taffy::prelude::{FromFr, FromLength, TaffyAuto, TaffyGridLine};
use taffy::{
    AlignContent, AlignItems, Display, GridPlacement, GridTemplateComponent, JustifyContent, Line,
    NodeId, Style, TrackSizingFunction,
};

use super::layout_style;
use super::{Builder, Context};
use crate::error::RenderError;
use crate::scene::Visual;
use crate::theme::file::node::{BoxSpec, NodeKind, NodeSpec, Span};

impl Builder<'_> {
    pub(super) fn frame(
        &mut self,
        spec: &NodeSpec,
        layout: &BoxSpec,
        ctx: Context,
        out: &mut Vec<NodeId>,
    ) -> Result<(), RenderError> {
        let NodeKind::Frame {
            direction,
            fill,
            radius,
            table,
            children,
        } = &spec.kind
        else {
            return Ok(());
        };
        let visual = match fill {
            Some(fill) => Visual::Fill {
                color: self.theme.color(fill),
                radius: radius * self.scale,
            },
            None => Visual::Group,
        };
        let base = layout_style::from_box(layout, self.scale);
        let node = match table {
            Some(table) => {
                let (cells, columns) = self.table_cells(children, ctx)?;
                let mut template = vec![GridTemplateComponent::AUTO; columns];
                template.push(GridTemplateComponent::from_fr(1.0));
                let style = Style {
                    display: Display::Grid,
                    grid_template_columns: template,
                    grid_auto_rows: vec![TrackSizingFunction::from_length(
                        table.row_height * self.scale,
                    )],
                    justify_content: Some(JustifyContent::START),
                    align_content: Some(AlignContent::START),
                    align_items: Some(AlignItems::START),
                    justify_items: Some(AlignItems::START),
                    ..base
                };
                self.scene.node(style, visual, &cells)?
            }
            None => {
                let mut nodes = Vec::with_capacity(children.len());
                for child in children {
                    self.node(child, ctx, &BoxSpec::default(), &mut nodes)?;
                }
                self.scene
                    .node(layout_style::flex(base, *direction), visual, &nodes)?
            }
        };
        out.push(node);
        Ok(())
    }

    /// 表格的格子（已放好行列）与列数。
    fn table_cells(
        &mut self,
        children: &[NodeSpec],
        ctx: Context,
    ) -> Result<(Vec<NodeId>, usize), RenderError> {
        let theme = self.theme;
        let mut cells = Vec::new();
        let mut columns = 0;
        for child in children {
            if child.when.as_deref().is_some_and(|when| !ctx.holds(when)) {
                continue;
            }
            let NodeKind::Repeat { bind, component } = &child.kind else {
                continue;
            };
            let (Some(rows), Some(component)) =
                (ctx.list(bind), theme.file().components.get(component))
            else {
                continue;
            };
            let NodeKind::Frame {
                children: row_cells,
                ..
            } = &component.kind
            else {
                continue;
            };
            for (i, row) in rows.iter().enumerate() {
                let row_ctx = ctx.with_row(row, i, rows.len());
                let grid_row = Line {
                    start: GridPlacement::from_line_index(i as i16 + 1),
                    end: GridPlacement::AUTO,
                };
                let mut column = 0;
                for cell in row_cells {
                    let mut built = Vec::with_capacity(1);
                    self.node(cell, row_ctx, &BoxSpec::default(), &mut built)?;
                    let span = cell.layout.span == Some(Span::Row);
                    for node in built {
                        let grid_column = if span {
                            Line {
                                start: GridPlacement::from_line_index(1),
                                end: GridPlacement::from_line_index(-1),
                            }
                        } else {
                            column += 1;
                            Line {
                                start: GridPlacement::from_line_index(column as i16),
                                end: GridPlacement::AUTO,
                            }
                        };
                        self.scene
                            .place(node, grid_row.clone(), grid_column, span)?;
                        cells.push(node);
                    }
                }
                columns = columns.max(column);
            }
        }
        Ok((cells, columns))
    }
}
