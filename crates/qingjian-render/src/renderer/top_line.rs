//! 顶部拼音行：各段按样式排开，光标叠在光标位置上，右侧是整句补全或临时状态。

use taffy::{NodeId, Rect, Style};

use super::metrics::Metrics;
use super::{CARET_WIDTH, Renderer, SENTENCE_GAP, nodes, styles};
use crate::error::RenderError;
use crate::frame::{Frame, Preedit, PreeditStyle};
use crate::scene::{Scene, Visual};

impl Renderer {
    /// 拼音行节点；帧里没有这一行时为 `None`。
    pub(super) fn top_line(
        &mut self,
        scene: &mut Scene,
        frame: &Frame,
        m: &Metrics,
    ) -> Result<Option<NodeId>, RenderError> {
        if !frame.has_top_line() {
            return Ok(None);
        }
        let mut children = Vec::with_capacity(2);
        if let Some(preedit) = &frame.preedit {
            children.push(self.preedit(scene, preedit, m)?);
        }
        // 整句补全：云朵 + 句子，颜色与本地候选区分；临时状态灰字、不带云朵
        if let Some((text, cloud)) = frame.trailing() {
            let mut parts = Vec::with_capacity(2);
            let color = if cloud {
                parts.push(nodes::cloud(scene, m, m.annotation_line_height())?);
                m.theme.colors.cloud
            } else {
                m.theme.colors.gloss
            };
            parts.push(nodes::text(
                scene,
                text,
                m.annotation_style(color),
                Rect::zero(),
            )?);
            let gap = if frame.preedit.is_some() {
                m.px(SENTENCE_GAP)
            } else {
                0.0
            };
            let style = Style {
                margin: styles::margin(0.0, 0.0, 0.0, gap),
                ..styles::row()
            };
            children.push(scene.node(style, Visual::Group, &parts)?);
        }
        let row_padding = m.row_padding();
        let style = Style {
            padding: styles::padding(row_padding, 0.0, row_padding, 0.0),
            ..styles::row()
        };
        scene.node(style, Visual::Group, &children).map(Some)
    }

    /// 拼音各段 + 行尾留出光标宽度；光标绝对定位在光标前文字的宽度处。
    fn preedit(
        &mut self,
        scene: &mut Scene,
        preedit: &Preedit,
        m: &Metrics,
    ) -> Result<NodeId, RenderError> {
        let caret_width = m.px(CARET_WIDTH);
        let line_height = m.annotation_line_height();
        let mut children = Vec::with_capacity(preedit.segments.len() + 2);
        let caret_x = self
            .text
            .measure(
                &preedit.before_cursor(),
                &m.annotation_style(m.theme.colors.gloss),
            )
            .width;
        for segment in &preedit.segments {
            let style = match segment.style {
                PreeditStyle::Typed => m.annotation_style(m.theme.colors.gloss),
                PreeditStyle::Rest => m.annotation_style(m.theme.colors.pos),
                PreeditStyle::Struck => m.annotation_style(m.theme.colors.pos).struck(),
            };
            children.push(nodes::text(scene, &segment.text, style, Rect::zero())?);
        }
        children.push(scene.node(styles::fixed(caret_width, 0.0), Visual::Group, &[])?);
        // 光标最后画，压在字形上面
        let mut caret = styles::absolute(Some(caret_x), None, Some(0.0), None);
        caret.size = styles::fixed(caret_width, line_height).size;
        children.push(scene.node(
            caret,
            Visual::Fill {
                color: m.theme.colors.text,
                radius: 0.0,
            },
            &[],
        )?);
        scene.node(styles::row(), Visual::Group, &children)
    }
}
