//! 渲染器：一帧 + 排布 + 主题 → 位图。排版与 macOS 壳的 `CandidateView` 一致：顶部拼音行，竖排一行一个候选、横排排成一行。
//!
//! 先按帧建一棵场景树（[`Scene`]），Taffy 算布局，再按树序画。内部全用像素：主题里的点数进来先乘缩放倍数。
//! 文字节点的盒子顶边就是行框顶边，字形在行高里垂直居中。

mod horizontal;
mod metrics;
mod nodes;
mod rendered;
mod status;
mod styles;
mod top_line;
mod vertical;

use taffy::{LengthPercentageAuto, NodeId, Size, Style};

use crate::canvas::Canvas;
use crate::color::Color;
use crate::error::RenderError;
use crate::fonts::FontLibrary;
use crate::frame::Frame;
use crate::layout::Layout;
use crate::scene::{Scene, Visual};
use crate::shadow::Shadow;
use crate::text::{TextPainter, TextStyle};
use crate::theme::{FontSpec, Theme};

use metrics::Metrics;

pub use rendered::Rendered;
pub use status::{RenderedStatus, StatusCell};

/// preedit 光标的宽度（点）。
const CARET_WIDTH: f32 = 1.5;

/// 云朵图标边长（点）。
const CLOUD_SIZE: f32 = 13.0;

/// 云朵与后面文字的间距（点）。
const CLOUD_GAP: f32 = 4.0;

/// preedit 与右侧整句补全之间的间距（点）。
const SENTENCE_GAP: f32 = 16.0;

/// 横排时序号与候选词之间的间距（点）。
const INDEX_GAP: f32 = 3.0;

/// 横排时高亮底色在候选两侧多出的宽度（点）。
const HIGHLIGHT_INSET: f32 = 5.0;

/// 光学字号（点）：20 pt 以下 CoreText 给系统字体用的就是这一档。
const OPTICAL_SIZE: f32 = 17.0;

/// 竖排候选窗口的最小宽度（点）。
const MIN_VERTICAL_WIDTH: f32 = 200.0;

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

    /// 画一帧。`scale` 是点 → 像素的倍数（Retina 为 2）；带 `shadow` 时位图四周留出阴影的边。
    pub fn render(
        &mut self,
        frame: &Frame,
        layout: Layout,
        theme: &Theme,
        scale: f32,
        shadow: Option<&Shadow>,
    ) -> Result<Rendered, RenderError> {
        let metrics = Metrics { theme, scale };
        let mut scene = Scene::new();
        let root = self.window(&mut scene, frame, layout, &metrics)?;
        let (content_width, content_height) = scene.layout(root, &mut self.text)?;
        let margin = shadow.map_or(0.0, |s| metrics.px(s.margin()));
        let width = (content_width + margin * 2.0).ceil();
        let height = (content_height + margin * 2.0).ceil();
        let mut canvas = Canvas::new(width as u32, height as u32)?;
        if let Some(shadow) = shadow
            && let Some(content) =
                tiny_skia::Rect::from_xywh(margin, margin, content_width, content_height)
        {
            shadow.paint(&mut canvas, content, metrics.corner_radius(), scale);
        }
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

    /// 每个字形用到的字族名，验证回退链用。
    pub fn trace_families(&mut self, text: &str, theme: &Theme) -> Vec<String> {
        let metrics = Metrics { theme, scale: 1.0 };
        self.text.trace_families(text, &metrics.text_style())
    }

    /// 窗口：圆角背景，内边距里纵向排拼音行与候选；竖排有最小宽度，高亮条绝对定位在窗口里。
    fn window(
        &mut self,
        scene: &mut Scene,
        frame: &Frame,
        layout: Layout,
        m: &Metrics,
    ) -> Result<NodeId, RenderError> {
        let padding = m.padding();
        let mut children = Vec::new();
        let top_line = self.top_line(scene, frame, m)?;
        let mut style = Style {
            padding: styles::padding(padding, padding, padding, padding),
            ..styles::column()
        };
        match layout {
            Layout::Vertical => {
                let top_height = if top_line.is_some() {
                    m.annotation_line_height() + m.row_padding() * 2.0
                } else {
                    0.0
                };
                children.extend(self.vertical_highlight(scene, frame, m, padding + top_height)?);
                children.extend(top_line);
                children.extend(self.vertical_body(scene, frame, m)?);
                // 竖排时候选都很短（没有译词）窗口会窄得难看，给个下限
                style.min_size = Size {
                    width: LengthPercentageAuto::length(m.px(MIN_VERTICAL_WIDTH)),
                    height: LengthPercentageAuto::auto(),
                };
            }
            Layout::Horizontal => {
                children.extend(top_line);
                children.extend(self.horizontal_body(scene, frame, m)?);
            }
        }
        scene.node(
            style,
            Visual::Fill {
                color: m.theme.colors.background,
                radius: m.corner_radius(),
            },
            &children,
        )
    }
}
