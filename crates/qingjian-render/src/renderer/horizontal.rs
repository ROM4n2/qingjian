//! 横排：候选排成一行（高亮条向两侧多出一点），页码靠行尾，高亮那个的译文在下面单独一行。

use taffy::{AlignSelf, Dimension, LengthPercentageAuto, NodeId, Size, Style};

use super::metrics::Metrics;
use super::{HIGHLIGHT_INSET, INDEX_GAP, Renderer, nodes, styles};
use crate::error::RenderError;
use crate::frame::Frame;
use crate::scene::{Scene, Visual};

impl Renderer {
    pub(super) fn horizontal_body(
        &mut self,
        scene: &mut Scene,
        frame: &Frame,
        m: &Metrics,
    ) -> Result<Vec<NodeId>, RenderError> {
        if frame.rows.is_empty() {
            return Ok(Vec::new());
        }
        let row_padding = m.row_padding();
        let small = row_padding + m.small_offset();
        let inset = m.px(HIGHLIGHT_INSET);
        let last = frame.rows.len() - 1;
        let mut items = Vec::with_capacity(frame.rows.len() + 1);
        for (i, row) in frame.rows.iter().enumerate() {
            let mut children = Vec::with_capacity(3);
            if Some(i) == frame.highlighted {
                // 高亮条比这一项左右各宽出 inset，压进项间距里
                let style = styles::absolute(Some(-inset), Some(-inset), Some(0.0), Some(0.0));
                children.push(scene.node(
                    style,
                    Visual::Fill {
                        color: m.theme.colors.highlight,
                        radius: m.corner_radius() / 2.0,
                    },
                    &[],
                )?);
            }
            children.push(nodes::text(
                scene,
                &row.index,
                m.index_style(),
                styles::margin(small, m.px(INDEX_GAP), 0.0, 0.0),
            )?);
            children.push(nodes::word(scene, m, row, styles::margin_top(row_padding))?);
            let mut style = styles::row();
            style.size.height = Dimension::length(m.row_height());
            // 最后一项右侧留出高亮条多出的那块
            if i == last {
                style.margin = styles::margin(0.0, inset, 0.0, 0.0);
            }
            items.push(scene.node(style, Visual::Group, &children)?);
        }
        if let Some(footer) = frame.footer.as_deref() {
            let mut margin = styles::margin_top(small);
            margin.left = LengthPercentageAuto::auto();
            items.push(nodes::text(scene, footer, m.index_style(), margin)?);
        }
        let style = Style {
            padding: styles::padding(0.0, 0.0, 0.0, inset),
            gap: Size {
                width: taffy::LengthPercentage::length(m.column_gap()),
                height: taffy::LengthPercentage::length(0.0),
            },
            align_self: Some(AlignSelf::STRETCH),
            ..styles::row()
        };
        let mut body = vec![scene.node(style, Visual::Group, &items)?];

        // 高亮候选的译文：从候选行第一项的位置开始；量宽时不算 inset（左加右减）
        if let Some(row) = frame.highlighted.and_then(|i| frame.rows.get(i))
            && !row.annotation.is_empty()
        {
            let style = Style {
                margin: styles::margin(row_padding / 2.0, -inset, 0.0, inset),
                padding: styles::padding(0.0, 0.0, row_padding / 2.0, 0.0),
                ..styles::row()
            };
            body.push(nodes::annotation(scene, m, &row.annotation, style)?);
        }
        Ok(body)
    }
}
