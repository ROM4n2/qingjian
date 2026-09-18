//! 画一个节点自己的画面（不含子节点）。

use super::draw_box::draw_box;
use super::{Icon, Visual};
use crate::canvas::Canvas;
use crate::cloud::draw_cloud;
use crate::gear::draw_gear;
use crate::text::TextPainter;

/// `rect` 是节点盒子（像素，画布坐标）。
pub(super) fn draw_visual(
    canvas: &mut Canvas,
    text: &mut TextPainter,
    visual: &Visual,
    rect: (f32, f32, f32, f32),
) {
    let (x, y, _, height) = rect;
    match visual {
        Visual::Box(paint) => draw_box(canvas, rect, paint),
        Visual::Text {
            text: content,
            style,
        } => {
            text.draw(canvas, content, style, x, y);
        }
        Visual::Icon { icon, size, color } => {
            let top = y + (height - size) / 2.0;
            match icon {
                Icon::Cloud => draw_cloud(canvas, x, top, *size, *color),
                Icon::Gear => draw_gear(canvas, x, top, *size, *color),
            }
        }
        Visual::Group => {}
    }
}
