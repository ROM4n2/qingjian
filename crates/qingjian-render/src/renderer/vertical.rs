//! 竖排：一行一个候选，序号 / 候选词 / 译文三列（grid，各列取各行最宽），页码在右下角，高亮条横跨整个窗口。

use taffy::prelude::{FromLength, TaffyAuto};
use taffy::{
    AlignContent, Display, GridTemplateComponent, JustifyContent, NodeId, Size, Style,
    TrackSizingFunction,
};

use super::metrics::Metrics;
use super::{Renderer, nodes, styles};
use crate::error::RenderError;
use crate::frame::Frame;
use crate::scene::{Scene, Visual};

impl Renderer {
    /// 候选表格与页码。
    pub(super) fn vertical_body(
        &mut self,
        scene: &mut Scene,
        frame: &Frame,
        m: &Metrics,
    ) -> Result<Vec<NodeId>, RenderError> {
        let mut body = Vec::with_capacity(2);
        if !frame.rows.is_empty() {
            body.push(self.vertical_grid(scene, frame, m)?);
        }
        if let Some(footer) = frame.footer.as_deref() {
            let mut style = Style {
                margin: styles::margin_top(m.row_padding()),
                ..Style::default()
            };
            style.align_self = Some(taffy::AlignSelf::END);
            let node = scene.node(
                style,
                Visual::Text {
                    text: footer.to_owned(),
                    style: m.index_style(),
                },
                &[],
            )?;
            body.push(node);
        }
        Ok(body)
    }

    /// 高亮条：绝对定位在窗口里，左右各缩进半个内边距；`top` 是候选表格顶边相对窗口的位置。
    pub(super) fn vertical_highlight(
        &mut self,
        scene: &mut Scene,
        frame: &Frame,
        m: &Metrics,
        top: f32,
    ) -> Result<Option<NodeId>, RenderError> {
        let Some(highlighted) = frame.highlighted.filter(|&i| i < frame.rows.len()) else {
            return Ok(None);
        };
        let row_height = m.row_height();
        let mut y = top;
        for _ in 0..highlighted {
            y += row_height;
        }
        let inset = m.padding() / 2.0;
        let mut style = styles::absolute(Some(inset), Some(inset), Some(y), None);
        style.size.height = taffy::Dimension::length(row_height);
        scene
            .node(
                style,
                Visual::Fill {
                    color: m.theme.colors.highlight,
                    radius: m.corner_radius() / 2.0,
                },
                &[],
            )
            .map(Some)
    }

    fn vertical_grid(
        &mut self,
        scene: &mut Scene,
        frame: &Frame,
        m: &Metrics,
    ) -> Result<NodeId, RenderError> {
        let has_annotation = frame
            .rows
            .iter()
            .any(|row| row.annotation.iter().any(|(s, _)| !s.is_empty()));
        let columns = if has_annotation { 3 } else { 2 };
        let row_padding = m.row_padding();
        let small = row_padding + m.small_offset();
        let mut cells = Vec::with_capacity(frame.rows.len() * columns);
        for row in &frame.rows {
            cells.push(nodes::text(
                scene,
                &row.index,
                m.index_style(),
                styles::margin_top(small),
            )?);
            cells.push(nodes::word(scene, m, row, styles::margin_top(row_padding))?);
            if has_annotation {
                let style = Style {
                    margin: styles::margin_top(small),
                    ..styles::row()
                };
                cells.push(nodes::annotation(scene, m, &row.annotation, style)?);
            }
        }
        let style = Style {
            display: Display::Grid,
            grid_template_columns: vec![GridTemplateComponent::AUTO; columns],
            grid_auto_rows: vec![TrackSizingFunction::from_length(m.row_height())],
            gap: Size {
                width: taffy::LengthPercentage::length(m.column_gap()),
                height: taffy::LengthPercentage::length(0.0),
            },
            justify_content: Some(JustifyContent::START),
            align_content: Some(AlignContent::START),
            align_items: Some(taffy::AlignItems::START),
            justify_items: Some(taffy::AlignItems::START),
            ..Style::default()
        };
        scene.node(style, Visual::Group, &cells)
    }
}
