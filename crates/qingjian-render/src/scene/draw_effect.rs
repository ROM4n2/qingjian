//! 画投影与内阴影：把节点自己的画面画进一张离屏图取 alpha 当形状，偏移、模糊、染色后合成。
//! 离屏图只取形状加模糊铺开的那一块，不是整张画布。

use super::draw_visual::draw_visual;
use super::{BoxPaint, Effect, EffectKind, Visual};
use crate::canvas::Canvas;
use crate::shadow;
use crate::text::TextPainter;

/// 画一个效果；`rect` 是节点盒子（像素，画布坐标）。
pub(super) fn draw_effect(
    canvas: &mut Canvas,
    text: &mut TextPainter,
    visual: &Visual,
    rect: (f32, f32, f32, f32),
    effect: &Effect,
) {
    let (x, y, width, height) = rect;
    let pad = shadow::reach(effect.blur) + effect.spread.abs();
    let (sx, sy) = match effect.kind {
        EffectKind::DropShadow => (x + effect.x, y + effect.y),
        EffectKind::InnerShadow => (x, y),
    };
    // 离屏区域：形状（投影按偏移挪过）四周加模糊铺开的宽度，夹在画布里
    let left = ((sx - pad).floor() as i32).max(0);
    let top = ((sy - pad).floor() as i32).max(0);
    let right = ((sx + width + pad).ceil() as i32).min(canvas.width() as i32);
    let bottom = ((sy + height + pad).ceil() as i32).min(canvas.height() as i32);
    if right <= left || bottom <= top {
        return;
    }
    let (w, h) = ((right - left) as u32, (bottom - top) as u32);
    let (ox, oy) = (left as f32, top as f32);
    let mask = match effect.kind {
        EffectKind::DropShadow => {
            let spread = effect.spread;
            let shape = (x + effect.x - ox, y + effect.y - oy, width, height);
            let Some(mut mask) = shape_alpha(text, visual, shape, spread, (w, h)) else {
                return;
            };
            shadow::blur(&mut mask, w as usize, h as usize, effect.blur);
            mask
        }
        EffectKind::InnerShadow => {
            let shape = (x - ox, y - oy, width, height);
            let Some(inside) = shape_alpha(text, visual, shape, 0.0, (w, h)) else {
                return;
            };
            // 形状外（按偏移挪过、按扩展缩过）为实，模糊后只留形状里的部分
            let hole = (x + effect.x - ox, y + effect.y - oy, width, height);
            let Some(mut mask) = shape_alpha(text, visual, hole, -effect.spread, (w, h)) else {
                return;
            };
            for value in &mut mask {
                *value = 255 - *value;
            }
            shadow::blur(&mut mask, w as usize, h as usize, effect.blur);
            for (value, inside) in mask.iter_mut().zip(&inside) {
                *value = ((u32::from(*value) * u32::from(*inside) + 127) / 255) as u8;
            }
            mask
        }
    };
    canvas.blend_mask(left, top, w, h, &mask, effect.color);
}

/// 节点画面的 alpha；盒子按 `spread` 外扩（负数内缩）。
fn shape_alpha(
    text: &mut TextPainter,
    visual: &Visual,
    rect: (f32, f32, f32, f32),
    spread: f32,
    size: (u32, u32),
) -> Option<Vec<u8>> {
    let mut layer = Canvas::new(size.0, size.1).ok()?;
    match visual {
        Visual::Box(paint) if spread != 0.0 => {
            let (x, y, width, height) = rect;
            let grown = (
                x - spread,
                y - spread,
                (width + spread * 2.0).max(0.0),
                (height + spread * 2.0).max(0.0),
            );
            let paint = BoxPaint {
                radius: if paint.radius > 0.0 {
                    (paint.radius + spread).max(0.0)
                } else {
                    0.0
                },
                ..paint.clone()
            };
            draw_visual(&mut layer, text, &Visual::Box(paint), grown);
        }
        _ => draw_visual(&mut layer, text, visual, rect),
    }
    Some(
        layer
            .into_pixmap()
            .pixels()
            .iter()
            .map(|pixel| pixel.alpha())
            .collect(),
    )
}
