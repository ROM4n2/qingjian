//! 悬浮状态条（Windows）：几格并排的小条 `[中 / 英][，。/ ,.][⚙]`，每格文字居中、格间一条细线，圆角背景加阴影。
//! macOS 用菜单栏状态项，没有这一块。

mod cell;
mod rendered;

pub use cell::StatusCell;
pub use rendered::RenderedStatus;

use taffy::{AlignItems, Dimension, Display, JustifyContent, NodeId, Style};

use super::metrics::Metrics;
use super::{Rendered, Renderer, styles};
use crate::canvas::Canvas;
use crate::error::RenderError;
use crate::scene::{Icon, Scene, Visual};
use crate::shadow::Shadow;
use crate::theme::Theme;

/// 齿轮图标边长（点）。
const GEAR_SIZE: f32 = 15.0;

/// 格间细线的宽度（点）。
const SEPARATOR_WIDTH: f32 = 1.0;

impl Renderer {
    /// 画状态条。每格宽 = 内容宽 + 两侧内边距，高 = 候选词行高 + 内边距；返回位图与各格右边界（供点击命中）。
    pub fn render_status(
        &mut self,
        cells: &[StatusCell],
        theme: &Theme,
        scale: f32,
        shadow: Option<&Shadow>,
    ) -> Result<RenderedStatus, RenderError> {
        let metrics = Metrics { theme, scale };
        let padding = metrics.padding();
        let mut widths: Vec<f32> = cells
            .iter()
            .map(|cell| self.status_cell_width(cell, &metrics) + padding * 2.0)
            .collect();
        let total: f32 = widths.iter().sum();
        let content_width = total.ceil();
        // 取整多出来的零头给最后一格，让最后一格的右边界正好是内容宽
        if let Some(last) = widths.last_mut() {
            *last += content_width - total;
        }
        let content_height = (metrics.text_line_height() + padding).ceil();

        let mut scene = Scene::new();
        let mut children = Vec::with_capacity(cells.len());
        for (i, (cell, width)) in cells.iter().zip(&widths).enumerate() {
            children.push(self.status_cell(
                &mut scene,
                cell,
                &metrics,
                i > 0,
                *width,
                content_height,
            )?);
        }
        let root = scene.node(
            styles::row(),
            Visual::Fill {
                color: theme.colors.background,
                radius: metrics.corner_radius(),
            },
            &children,
        )?;
        scene.layout(root, &mut self.text)?;

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
        let edges = widths
            .iter()
            .scan(0.0, |x, width| {
                *x += width;
                Some(*x)
            })
            .collect();
        Ok(RenderedStatus {
            rendered: Rendered {
                pixmap: canvas.into_pixmap(),
                content_x: margin as u32,
                content_y: margin as u32,
                content_width: content_width as u32,
                content_height: content_height as u32,
                scale,
            },
            cell_edges: edges,
        })
    }

    /// 一格内容的宽度（像素，不含内边距）。
    fn status_cell_width(&mut self, cell: &StatusCell, m: &Metrics) -> f32 {
        match cell {
            StatusCell::Text { text, .. } => self.text.measure(text, &m.text_style()).width,
            StatusCell::Gear => m.px(GEAR_SIZE),
        }
    }

    /// 一格：内容在格里居中；不是第一格时左边画一条上下各缩进半个内边距的细线。
    fn status_cell(
        &mut self,
        scene: &mut Scene,
        cell: &StatusCell,
        m: &Metrics,
        separator: bool,
        width: f32,
        height: f32,
    ) -> Result<NodeId, RenderError> {
        let mut children = Vec::with_capacity(2);
        if separator {
            let inset = m.padding() / 2.0;
            let mut style = styles::absolute(Some(0.0), None, Some(inset), Some(inset));
            style.size.width = Dimension::length(m.px(SEPARATOR_WIDTH));
            children.push(scene.node(
                style,
                Visual::Fill {
                    color: m.theme.colors.pos,
                    radius: 0.0,
                },
                &[],
            )?);
        }
        let content = match cell {
            StatusCell::Text { text, emphasized } => {
                let color = if *emphasized {
                    m.theme.colors.cloud
                } else {
                    m.theme.colors.gloss
                };
                Visual::Text {
                    text: text.clone(),
                    style: m.style(m.theme.text_font, color),
                }
            }
            StatusCell::Gear => Visual::Icon {
                icon: Icon::Gear,
                size: m.px(GEAR_SIZE),
                color: m.theme.colors.gloss,
            },
        };
        let content_style = match cell {
            StatusCell::Text { .. } => Style::default(),
            StatusCell::Gear => styles::fixed(m.px(GEAR_SIZE), m.px(GEAR_SIZE)),
        };
        children.push(scene.node(content_style, content, &[])?);
        let style = Style {
            display: Display::Flex,
            justify_content: Some(JustifyContent::CENTER),
            align_items: Some(AlignItems::CENTER),
            ..styles::fixed(width, height)
        };
        scene.node(style, Visual::Group, &children)
    }
}

#[cfg(test)]
mod tests {
    use super::StatusCell;
    use crate::fonts::FontLibrary;
    use crate::renderer::Renderer;
    use crate::shadow::Shadow;
    use crate::theme::Theme;

    #[test]
    fn cells_have_increasing_edges_ending_at_content_width() {
        // 没有系统字体的环境（CI 容器）跳过
        let Ok(library) = FontLibrary::system("zh-CN") else {
            return;
        };
        let mut renderer = Renderer::new(library);
        let cells = [
            StatusCell::text("中 · 小鹤", true),
            StatusCell::text(",.", false),
            StatusCell::Gear,
        ];
        let out = renderer
            .render_status(&cells, &Theme::light(), 2.0, Some(&Shadow::mac_panel()))
            .unwrap();
        assert_eq!(out.cell_edges.len(), 3);
        assert!(out.cell_edges.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            out.cell_edges.last().map(|edge| edge.round() as u32),
            Some(out.rendered.content_width)
        );
        assert!(out.rendered.pixmap.width() > out.rendered.content_width);
        assert!(out.rendered.content_x > 0);
    }
}
