//! 竖排、横排、拼音行共用的几种节点：一段文字、候选词（可带云朵）、一串译文片段。

use taffy::{NodeId, Rect, Style};

use super::metrics::Metrics;
use super::styles;
use crate::color::Color;
use crate::error::RenderError;
use crate::frame::{Row, Tone};
use crate::scene::{Icon, Scene, Visual};
use crate::text::TextStyle;

/// 一段文字，带外边距。
pub(super) fn text(
    scene: &mut Scene,
    content: &str,
    style: TextStyle,
    margin: Rect<taffy::LengthPercentageAuto>,
) -> Result<NodeId, RenderError> {
    scene.node(
        Style {
            margin,
            ..Style::default()
        },
        Visual::Text {
            text: content.to_owned(),
            style,
        },
        &[],
    )
}

/// 云朵图标：占 `cloud_size` 宽、`line_height` 高，后面留 `cloud_gap`。
pub(super) fn cloud(
    scene: &mut Scene,
    m: &Metrics,
    line_height: f32,
) -> Result<NodeId, RenderError> {
    let mut style = styles::fixed(m.cloud_size(), line_height);
    style.margin = styles::margin(0.0, m.cloud_gap(), 0.0, 0.0);
    scene.node(
        style,
        Visual::Icon {
            icon: Icon::Cloud,
            size: m.cloud_size(),
            color: m.theme.colors.cloud,
        },
        &[],
    )
}

/// 候选词本体：云端词前带云朵、换颜色。
pub(super) fn word(
    scene: &mut Scene,
    m: &Metrics,
    row: &Row,
    margin: Rect<taffy::LengthPercentageAuto>,
) -> Result<NodeId, RenderError> {
    let mut children = Vec::with_capacity(2);
    let color = if row.cloud {
        children.push(cloud(scene, m, m.text_line_height())?);
        m.theme.colors.cloud
    } else {
        m.theme.colors.text
    };
    let style = m.style(m.theme.text_font, color);
    children.push(text(scene, &row.text, style, Rect::zero())?);
    scene.node(
        Style {
            margin,
            ..styles::row()
        },
        Visual::Group,
        &children,
    )
}

/// 一串译文片段，各按深浅着色。
pub(super) fn annotation(
    scene: &mut Scene,
    m: &Metrics,
    segments: &[(String, Tone)],
    style: Style,
) -> Result<NodeId, RenderError> {
    let children = segments
        .iter()
        .map(|(segment, tone)| {
            let color: Color = m.tone_color(*tone);
            text(scene, segment, m.annotation_style(color), Rect::zero())
        })
        .collect::<Result<Vec<_>, _>>()?;
    scene.node(style, Visual::Group, &children)
}
