//! 画一个盒子：填充（纯色 / 渐变 / 图片）再画内侧边框。纯色走原来的画法，保证与改动前逐像素一致。

use tiny_skia::{
    FilterQuality, GradientStop, LinearGradient, Mask, Pattern, Point, RadialGradient, Rect,
    Shader, SpreadMode, Transform,
};

use super::box_paint::BoxPaint;
use super::fill::Fill;
use crate::canvas::{Canvas, round_rect};
use crate::color::Color;

/// 在 `(x, y, width, height)`（像素）画 `paint`。
pub(super) fn draw_box(canvas: &mut Canvas, rect: (f32, f32, f32, f32), paint: &BoxPaint) {
    let (x, y, width, height) = rect;
    let radius = paint.radius;
    match &paint.fill {
        Some(Fill::Solid(color)) if radius > 0.0 => {
            canvas.fill_round_rect(x, y, width, height, radius, *color);
        }
        Some(Fill::Solid(color)) => canvas.fill_rect(x, y, width, height, *color),
        Some(Fill::Linear { angle, stops }) => {
            if let (Some(path), Some(shader)) = (
                round_rect(x, y, width, height, radius),
                linear(rect, *angle, stops),
            ) {
                canvas.fill_path_with(&path, shader, None);
            }
        }
        Some(Fill::Radial { center, stops }) => {
            if let (Some(path), Some(shader)) = (
                round_rect(x, y, width, height, radius),
                radial(rect, *center, stops),
            ) {
                canvas.fill_path_with(&path, shader, None);
            }
        }
        Some(Fill::Image {
            pixmap,
            slice,
            pixels_per_px,
        }) => {
            let mask = (radius > 0.0)
                .then(|| round_rect(x, y, width, height, radius))
                .flatten()
                .and_then(|path| canvas.mask(&path));
            match slice {
                Some(slice) => {
                    nine_slice(canvas, rect, pixmap, *slice, *pixels_per_px, mask.as_ref())
                }
                None => stretch(canvas, rect, pixmap, mask.as_ref()),
            }
        }
        None => {}
    }
    if let Some((line, color)) = paint.border
        && line > 0.0
        && let Some(path) = round_rect(
            x + line / 2.0,
            y + line / 2.0,
            width - line,
            height - line,
            (radius - line / 2.0).max(0.0),
        )
    {
        canvas.stroke_path(&path, line, color);
    }
}

fn gradient_stops(stops: &[(Color, f32)]) -> Vec<GradientStop> {
    stops
        .iter()
        .map(|(color, position)| GradientStop::new(*position, color.to_skia()))
        .collect()
}

/// CSS 式线性渐变：渐变线过盒子中心，长度让两端正好碰到对角。
fn linear(
    rect: (f32, f32, f32, f32),
    angle: f32,
    stops: &[(Color, f32)],
) -> Option<Shader<'static>> {
    let (x, y, width, height) = rect;
    let radians = angle.to_radians();
    let (dx, dy) = (radians.sin(), -radians.cos());
    let half = (width * dx.abs() + height * dy.abs()) / 2.0;
    let (cx, cy) = (x + width / 2.0, y + height / 2.0);
    LinearGradient::new(
        Point::from_xy(cx - dx * half, cy - dy * half),
        Point::from_xy(cx + dx * half, cy + dy * half),
        gradient_stops(stops),
        SpreadMode::Pad,
        Transform::identity(),
    )
}

/// 径向渐变：圆心按比例，半径到最远的角。
fn radial(
    rect: (f32, f32, f32, f32),
    center: [f32; 2],
    stops: &[(Color, f32)],
) -> Option<Shader<'static>> {
    let (x, y, width, height) = rect;
    let (cx, cy) = (x + width * center[0], y + height * center[1]);
    let radius = [
        (x, y),
        (x + width, y),
        (x, y + height),
        (x + width, y + height),
    ]
    .into_iter()
    .map(|(px, py)| ((px - cx).powi(2) + (py - cy).powi(2)).sqrt())
    .fold(0.0_f32, f32::max);
    let center = Point::from_xy(cx, cy);
    RadialGradient::new(
        center,
        0.0,
        center,
        radius,
        gradient_stops(stops),
        SpreadMode::Pad,
        Transform::identity(),
    )
}

/// 整张图拉伸到盒子。
fn stretch(
    canvas: &mut Canvas,
    rect: (f32, f32, f32, f32),
    pixmap: &tiny_skia::Pixmap,
    mask: Option<&Mask>,
) {
    let (x, y, width, height) = rect;
    let (image_width, image_height) = (pixmap.width() as f32, pixmap.height() as f32);
    let transform = Transform::from_row(width / image_width, 0.0, 0.0, height / image_height, x, y);
    fill_cell(canvas, (x, y, width, height), pixmap, transform, mask);
}

/// 九宫格：四角原样（按倍数缩放），四边沿一个方向拉伸，中间两个方向拉伸。
fn nine_slice(
    canvas: &mut Canvas,
    rect: (f32, f32, f32, f32),
    pixmap: &tiny_skia::Pixmap,
    slice: [f32; 4],
    pixels_per_px: f32,
    mask: Option<&Mask>,
) {
    let (x, y, width, height) = rect;
    let (image_width, image_height) = (pixmap.width() as f32, pixmap.height() as f32);
    let [top, right, bottom, left] = slice;
    let per = pixels_per_px.max(f32::EPSILON);
    // 源（图片像素）与目标（画布像素）的四条分割线；目标四角按倍数，放不下时压缩到盒子里。
    // 内部两条目标分割线取整像素：相邻两块各自抗锯齿会在接缝留一道 alpha 不满的缝，底下有投影就透出来
    let source_x = [0.0, left, image_width - right, image_width];
    let source_y = [0.0, top, image_height - bottom, image_height];
    let shrink_x = ((left + right) / per / width).max(1.0);
    let shrink_y = ((top + bottom) / per / height).max(1.0);
    let target_x = [
        x,
        (x + left / per / shrink_x).round(),
        (x + width - right / per / shrink_x).round(),
        x + width,
    ];
    let target_y = [
        y,
        (y + top / per / shrink_y).round(),
        (y + height - bottom / per / shrink_y).round(),
        y + height,
    ];
    for row in 0..3 {
        for column in 0..3 {
            let (sx0, sx1) = (source_x[column], source_x[column + 1]);
            let (sy0, sy1) = (source_y[row], source_y[row + 1]);
            let (tx0, tx1) = (target_x[column], target_x[column + 1]);
            let (ty0, ty1) = (target_y[row], target_y[row + 1]);
            if sx1 <= sx0 || sy1 <= sy0 || tx1 <= tx0 || ty1 <= ty0 {
                continue;
            }
            let scale_x = (tx1 - tx0) / (sx1 - sx0);
            let scale_y = (ty1 - ty0) / (sy1 - sy0);
            let transform = Transform::from_row(
                scale_x,
                0.0,
                0.0,
                scale_y,
                tx0 - sx0 * scale_x,
                ty0 - sy0 * scale_y,
            );
            fill_cell(
                canvas,
                (tx0, ty0, tx1 - tx0, ty1 - ty0),
                pixmap,
                transform,
                mask,
            );
        }
    }
}

/// 用图片图案填一块矩形。
fn fill_cell(
    canvas: &mut Canvas,
    cell: (f32, f32, f32, f32),
    pixmap: &tiny_skia::Pixmap,
    transform: Transform,
    mask: Option<&Mask>,
) {
    let (x, y, width, height) = cell;
    let Some(rect) = Rect::from_xywh(x, y, width, height) else {
        return;
    };
    let shader = Pattern::new(
        pixmap.as_ref(),
        SpreadMode::Pad,
        FilterQuality::Bilinear,
        1.0,
        transform,
    );
    canvas.fill_path_with(&tiny_skia::PathBuilder::from_rect(rect), shader, mask);
}
