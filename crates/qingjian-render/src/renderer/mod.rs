//! 渲染器：一帧 + 排布 + 主题 → 位图。排版与 macOS 壳的 `CandidateView` 一致：顶部拼音行，竖排一行一个候选、横排排成一行。
//!
//! 先按帧建一棵场景树（[`Scene`]），Taffy 算布局，再按树序画。内部全用像素：主题里的点数进来先乘缩放倍数。
//! 文字节点的盒子顶边就是行框顶边，字形在行高里垂直居中。

mod build;
mod rendered;
mod status;

use crate::canvas::Canvas;
use crate::color::Color;
use crate::error::RenderError;
use crate::fonts::FontLibrary;
use crate::frame::Frame;
use crate::layout::Layout;
use crate::scene::Scene;
use crate::text::{TextPainter, TextStyle};
use crate::theme::{FontSpec, Theme};

use build::Builder;

pub use rendered::Rendered;
pub use status::{RenderedStatus, StatusCell};

/// 光学字号（点）：20 pt 以下 CoreText 给系统字体用的就是这一档。
const OPTICAL_SIZE: f32 = 17.0;

/// 候选词的文字样式名：状态条之外只有核对字体回退时直接用到。
const CANDIDATE_FONT: &str = "candidate";

pub struct Renderer {
    /// 文字测绘。
    text: TextPainter,
}

impl Renderer {
    pub fn new(library: FontLibrary) -> Self {
        let mut text = TextPainter::new(library);
        text.set_optical_size(Some(OPTICAL_SIZE));
        Self { text }
    }

    /// 画一帧。`scale` 是点 → 像素的倍数（Retina 为 2）；窗口根节点有投影时位图四周留出投影的边。
    pub fn render(
        &mut self,
        frame: &Frame,
        layout: Layout,
        theme: &Theme,
        scale: f32,
    ) -> Result<Rendered, RenderError> {
        let mut scene = Scene::new();
        let mut builder = Builder {
            scene: &mut scene,
            theme,
            scale,
            text: &mut self.text,
        };
        let root = builder.window(frame, layout)?;
        let (content_width, content_height) = scene.layout(root, &mut self.text)?;
        let margin = scene.overhang(root).ceil();
        let width = (content_width + margin * 2.0).ceil();
        let height = (content_height + margin * 2.0).ceil();
        let mut canvas = Canvas::new(width as u32, height as u32)?;
        scene.paint(root, &mut canvas, &mut self.text, margin, margin)?;
        Ok(Rendered {
            pixmap: canvas.into_pixmap(),
            content_x: margin as u32,
            content_y: margin as u32,
            content_width: content_width.ceil() as u32,
            content_height: content_height.ceil() as u32,
            scale,
        })
    }

    /// 量一段文字在 `size` 点字号下的宽度（点），与原生排版的数值对照用。
    pub fn measure_points(&mut self, text: &str, size: f32) -> f32 {
        let style = TextStyle::new(FontSpec::new(size, size), size, Color::rgb(0, 0, 0), 1.0);
        self.text.measure(text, &style).width
    }

    /// 每个字形用到的字族名（按候选词的字号），验证回退链用。
    pub fn trace_families(&mut self, text: &str, theme: &Theme) -> Vec<String> {
        let font = theme.font(CANDIDATE_FONT);
        let style = TextStyle::new(font, font.size, Color::rgb(0, 0, 0), theme.text_gamma());
        self.text.trace_families(text, &style)
    }
}
