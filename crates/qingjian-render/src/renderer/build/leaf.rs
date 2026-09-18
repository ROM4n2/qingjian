//! 叶子节点：文字、图标、拼音行、译文。绑定取不到数据时不产出节点。

use taffy::{Dimension, NodeId, Style};

use super::layout_style;
use super::{Builder, Context};
use crate::error::RenderError;
use crate::frame::{Preedit, PreeditStyle, Tone};
use crate::scene::Visual;
use crate::theme::file::node::{BoxSpec, Direction, NodeKind, NodeSpec};

impl Builder<'_> {
    pub(super) fn leaf(
        &mut self,
        spec: &NodeSpec,
        layout: &BoxSpec,
        ctx: Context,
    ) -> Result<Option<NodeId>, RenderError> {
        let style = layout_style::from_box(layout, self.scale);
        let node = match &spec.kind {
            NodeKind::Text {
                bind,
                text,
                font,
                color,
                stroke,
            } => {
                let content = match (bind, text) {
                    (Some(bind), _) => ctx.text(bind),
                    (None, Some(text)) => Some(text.as_str()),
                    (None, None) => None,
                };
                let Some(content) = content else {
                    return Ok(None);
                };
                let visual = Visual::Text {
                    text: content.to_owned(),
                    style: self
                        .text_style(font, self.color(color, ctx))
                        .stroked(self.stroke(stroke.as_ref(), ctx)),
                };
                self.scene.node(style, visual, &[])?
            }
            NodeKind::Icon { icon, size, color } => {
                let size = size * self.scale;
                let mut style = style;
                if layout.width.is_none() {
                    style.size.width = Dimension::length(size);
                }
                if layout.height.is_none() {
                    style.size.height = Dimension::length(size);
                }
                let visual = Visual::Icon {
                    icon: *icon,
                    size,
                    color: self.color(color, ctx),
                };
                self.scene.node(style, visual, &[])?
            }
            NodeKind::Preedit { .. } => {
                let Some(preedit) = &ctx.frame.preedit else {
                    return Ok(None);
                };
                self.preedit(spec, preedit, style, ctx)?
            }
            NodeKind::Annotation {
                bind,
                font,
                gloss,
                fresh,
                faint,
                stroke,
            } => {
                let Some(segments) = ctx.annotation(bind) else {
                    return Ok(None);
                };
                let stroke = self.stroke(stroke.as_ref(), ctx);
                let mut children = Vec::with_capacity(segments.len());
                for (segment, tone) in segments {
                    let color = match tone {
                        Tone::Gloss => gloss,
                        Tone::Fresh => fresh,
                        Tone::Faint => faint,
                    };
                    let visual = Visual::Text {
                        text: segment.clone(),
                        style: self
                            .text_style(font, self.color(color, ctx))
                            .stroked(stroke),
                    };
                    children.push(self.scene.node(Style::default(), visual, &[])?);
                }
                self.scene.node(
                    layout_style::flex(style, Direction::Row),
                    Visual::Group,
                    &children,
                )?
            }
            NodeKind::Frame { .. } | NodeKind::Use { .. } | NodeKind::Repeat { .. } => {
                return Ok(None);
            }
        };
        self.apply_layer(node, layout, ctx);
        Ok(Some(node))
    }

    /// 拼音各段 + 行尾留出光标宽度；光标绝对定位在光标前文字的宽度处，最后画、压在字形上。
    fn preedit(
        &mut self,
        spec: &NodeSpec,
        preedit: &Preedit,
        style: Style,
        ctx: Context,
    ) -> Result<NodeId, RenderError> {
        let NodeKind::Preedit {
            font,
            typed,
            rest,
            struck,
            caret,
            stroke,
        } = &spec.kind
        else {
            return self.scene.node(style, Visual::Group, &[]);
        };
        let stroke = self.stroke(stroke.as_ref(), ctx);
        let typed_style = self
            .text_style(font, self.color(typed, ctx))
            .stroked(stroke);
        let mut children = Vec::with_capacity(preedit.segments.len() + 2);
        for segment in &preedit.segments {
            let text_style = match segment.style {
                PreeditStyle::Typed => typed_style,
                PreeditStyle::Rest => self.text_style(font, self.color(rest, ctx)).stroked(stroke),
                PreeditStyle::Struck => self
                    .text_style(font, self.color(struck, ctx))
                    .stroked(stroke)
                    .struck(),
            };
            let visual = Visual::Text {
                text: segment.text.clone(),
                style: text_style,
            };
            children.push(self.scene.node(Style::default(), visual, &[])?);
        }
        let caret_width = caret.width * self.scale;
        let spacer = Style {
            size: taffy::Size {
                width: Dimension::length(caret_width),
                height: Dimension::length(0.0),
            },
            ..Style::default()
        };
        children.push(self.scene.node(spacer, Visual::Group, &[])?);
        let caret_x = self
            .text
            .measure(&preedit.before_cursor(), &typed_style)
            .width;
        let caret_style = Style {
            position: taffy::Position::Absolute,
            inset: taffy::Rect {
                left: taffy::LengthPercentageAuto::length(caret_x),
                right: taffy::LengthPercentageAuto::auto(),
                top: taffy::LengthPercentageAuto::length(0.0),
                bottom: taffy::LengthPercentageAuto::auto(),
            },
            size: taffy::Size {
                width: Dimension::length(caret_width),
                height: Dimension::length(typed_style.line_height),
            },
            ..Style::default()
        };
        let visual = Visual::solid(self.color(&caret.color, ctx), 0.0);
        children.push(self.scene.node(caret_style, visual, &[])?);
        self.scene.node(
            layout_style::flex(style, Direction::Row),
            Visual::Group,
            &children,
        )
    }
}
