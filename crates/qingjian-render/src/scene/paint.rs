//! 按树序画：先画节点自己（投影垫底、内阴影压在填充上；自己不画的容器取子节点当阴影形状），再画子节点；坐标是父节点位置加布局给的相对位置。
//! 不透明度小于 1 的节点连同子树先画到离屏图层，再按不透明度合成。

use taffy::NodeId;

use super::draw_effect::draw_effect;
use super::draw_visual::{draw_shape, draw_visual};
use super::{EffectKind, Scene, SceneNode, Visual};
use crate::canvas::Canvas;
use crate::error::RenderError;
use crate::text::TextPainter;

impl Scene {
    /// 把 `root` 画到画布上，`(x, y)` 是根节点左上角在画布里的像素坐标。
    pub(crate) fn paint(
        &self,
        root: NodeId,
        canvas: &mut Canvas,
        text: &mut TextPainter,
        x: f32,
        y: f32,
    ) -> Result<(), RenderError> {
        let opacity = self
            .tree
            .get_node_context(root)
            .map_or(1.0, |node| node.opacity);
        if opacity >= 1.0 {
            return self.paint_subtree(root, canvas, text, x, y);
        }
        if opacity <= 0.0 {
            return Ok(());
        }
        let mut layer = Canvas::new(canvas.width(), canvas.height())?;
        self.paint_subtree(root, &mut layer, text, x, y)?;
        canvas.draw_layer(&layer.into_pixmap(), opacity);
        Ok(())
    }

    fn paint_subtree(
        &self,
        root: NodeId,
        canvas: &mut Canvas,
        text: &mut TextPainter,
        x: f32,
        y: f32,
    ) -> Result<(), RenderError> {
        let layout = self.tree.layout(root)?;
        let (x, y) = (x + layout.location.x, y + layout.location.y);
        let rect = (x, y, layout.size.width, layout.size.height);
        if let Some(SceneNode {
            visual, effects, ..
        }) = self.tree.get_node_context(root)
        {
            let children = self.tree.children(root)?;
            for kind in [EffectKind::DropShadow, EffectKind::InnerShadow] {
                if kind == EffectKind::InnerShadow {
                    draw_visual(canvas, text, visual, rect);
                }
                for effect in effects.iter().filter(|effect| effect.kind == kind) {
                    // 自己不画东西的容器（译文、拼音行）拿子节点当形状
                    let mut shape = |layer: &mut Canvas, (x, y, width, height), spread| {
                        match visual {
                            Visual::Group => {
                                for &child in &children {
                                    self.paint(child, layer, text, x, y)?;
                                }
                            }
                            _ => draw_shape(layer, text, visual, (x, y, width, height), spread),
                        }
                        Ok(())
                    };
                    draw_effect(canvas, rect, effect, &mut shape)?;
                }
            }
        }
        for child in self.tree.children(root)? {
            self.paint(child, canvas, text, x, y)?;
        }
        Ok(())
    }
}
