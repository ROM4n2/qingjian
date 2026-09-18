//! 画投影与内阴影：把节点的形状画进一张离屏图取 alpha，偏移、模糊、染色后合成。
//! 离屏图只取形状加模糊铺开的那一块，不是整张画布。形状怎么画由调用方给（节点自己的画面，或容器的子节点）。

use super::{Effect, EffectKind};
use crate::canvas::Canvas;
use crate::error::RenderError;
use crate::shadow;

/// 在离屏图上画形状：盒子在 `rect`（离屏图坐标），按 `spread` 外扩（负数内缩）。
pub(super) type DrawShape<'a> =
    dyn FnMut(&mut Canvas, (f32, f32, f32, f32), f32) -> Result<(), RenderError> + 'a;

/// 画一个效果；`rect` 是节点盒子（像素，画布坐标）。
pub(super) fn draw_effect(
    canvas: &mut Canvas,
    rect: (f32, f32, f32, f32),
    effect: &Effect,
    shape: &mut DrawShape,
) -> Result<(), RenderError> {
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
        return Ok(());
    }
    let (w, h) = ((right - left) as u32, (bottom - top) as u32);
    let (ox, oy) = (left as f32, top as f32);
    let shifted = (x + effect.x - ox, y + effect.y - oy, width, height);
    let mask = match effect.kind {
        EffectKind::DropShadow => {
            let mut mask = alpha(shape, shifted, effect.spread, (w, h))?;
            shadow::blur(&mut mask, w as usize, h as usize, effect.blur);
            mask
        }
        EffectKind::InnerShadow => {
            let inside = alpha(shape, (x - ox, y - oy, width, height), 0.0, (w, h))?;
            // 形状外（按偏移挪过、按扩展缩过）为实，模糊后只留形状里的部分
            let mut mask = alpha(shape, shifted, -effect.spread, (w, h))?;
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
    Ok(())
}

/// 形状画进一张 `size` 大的离屏图，取 alpha。
fn alpha(
    shape: &mut DrawShape,
    rect: (f32, f32, f32, f32),
    spread: f32,
    size: (u32, u32),
) -> Result<Vec<u8>, RenderError> {
    let mut layer = Canvas::new(size.0, size.1)?;
    shape(&mut layer, rect, spread)?;
    Ok(layer
        .into_pixmap()
        .pixels()
        .iter()
        .map(|pixel| pixel.alpha())
        .collect())
}
