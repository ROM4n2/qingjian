//! 实例化：主题模板 + 一帧数据 → 场景树。节点按 `when` 取舍、按 `bind` 填数据，`use` 展开组件、`repeat` 按列表展开。
//!
//! 一个模板节点可能产出零个（条件不成立、绑定为空）或多个（`repeat`）场景节点，所以都往 `out` 里推。

mod context;
mod frame;
mod layout_style;
mod leaf;
mod row;

use taffy::NodeId;

use crate::error::RenderError;
use crate::frame::Frame;
use crate::layout::Layout;
use crate::scene::{Scene, Visual};
use crate::text::{TextPainter, TextStyle};
use crate::theme::Theme;
use crate::theme::file::ColorSpec;
use crate::theme::file::node::{BoxSpec, NodeKind, NodeSpec};

use context::Context;

pub(super) struct Builder<'a> {
    pub(super) scene: &'a mut Scene,

    pub(super) theme: &'a Theme,

    /// 点 → 像素。
    pub(super) scale: f32,

    /// 量光标位置用。
    pub(super) text: &'a mut TextPainter,
}

impl Builder<'_> {
    /// 候选窗口的根节点与它的圆角（像素，给阴影用）。
    pub(super) fn window(
        &mut self,
        frame: &Frame,
        layout: Layout,
    ) -> Result<(NodeId, f32), RenderError> {
        let windows = &self.theme.file().windows;
        let spec = match layout {
            Layout::Vertical => &windows.vertical,
            Layout::Horizontal => &windows.horizontal,
        };
        let radius = match &spec.kind {
            NodeKind::Frame { radius, .. } => radius * self.scale,
            _ => 0.0,
        };
        let mut out = Vec::with_capacity(1);
        self.node(spec, Context::new(frame), &BoxSpec::default(), &mut out)?;
        let root = match out.first() {
            Some(&root) => root,
            None => self
                .scene
                .node(taffy::Style::default(), Visual::Group, &[])?,
        };
        Ok((root, radius))
    }

    /// 实例化一个模板节点；`over` 是外层 `use` 写的盒子属性，盖过这个节点自己的。
    fn node(
        &mut self,
        spec: &NodeSpec,
        ctx: Context,
        over: &BoxSpec,
        out: &mut Vec<NodeId>,
    ) -> Result<(), RenderError> {
        if spec.when.as_deref().is_some_and(|when| !ctx.holds(when)) {
            return Ok(());
        }
        let layout = spec.layout.overridden_by(over);
        let theme = self.theme;
        match &spec.kind {
            NodeKind::Frame { .. } => self.frame(spec, &layout, ctx, out),
            NodeKind::Use { component } => match theme.file().components.get(component) {
                Some(component) => self.node(component, ctx, &layout, out),
                None => Ok(()),
            },
            NodeKind::Repeat { bind, component } => {
                let (Some(rows), Some(component)) =
                    (ctx.list(bind), theme.file().components.get(component))
                else {
                    return Ok(());
                };
                for (i, row) in rows.iter().enumerate() {
                    let row_ctx = ctx.with_row(row, i, rows.len());
                    self.node(component, row_ctx, &layout, out)?;
                }
                Ok(())
            }
            _ => {
                if let Some(node) = self.leaf(spec, &layout, ctx)? {
                    out.push(node);
                }
                Ok(())
            }
        }
    }

    /// 盒子属性里写了不透明度就设上（作用于整棵子树）。
    fn apply_opacity(&mut self, node: NodeId, layout: &BoxSpec) {
        if let Some(opacity) = layout.opacity
            && opacity < 1.0
        {
            self.scene.set_opacity(node, opacity);
        }
    }

    /// 节点颜色：条件写法按当前数据取分支，再按外观取值。
    fn color(&self, spec: &ColorSpec, ctx: Context) -> crate::color::Color {
        let color = match spec {
            ColorSpec::Fixed(color) => color,
            ColorSpec::Switch {
                condition,
                then,
                otherwise,
            } => {
                if ctx.holds(condition) {
                    then
                } else {
                    otherwise
                }
            }
        };
        self.theme.color(color)
    }

    /// 命名文字样式按倍数换成像素、配上颜色与当前外观的 gamma。
    fn text_style(&self, font: &str, color: crate::color::Color) -> TextStyle {
        let spec = self.theme.font(font);
        TextStyle::new(
            spec.scaled(self.scale),
            spec.size,
            color,
            self.theme.text_gamma(),
        )
    }
}
