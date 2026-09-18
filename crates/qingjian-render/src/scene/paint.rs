//! 按树序画：先画节点自己，再画子节点；坐标是父节点位置加布局给的相对位置。
//! 不透明度小于 1 的节点连同子树先画到离屏图层，再按不透明度合成。

use taffy::NodeId;

use super::draw_box::draw_box;
use super::{Icon, Scene, SceneNode, Visual};
use crate::canvas::Canvas;
use crate::cloud::draw_cloud;
use crate::error::RenderError;
use crate::gear::draw_gear;
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
        let (width, height) = (layout.size.width, layout.size.height);
        match self.tree.get_node_context(root) {
            Some(SceneNode {
                visual: Visual::Box(paint),
                ..
            }) => draw_box(canvas, (x, y, width, height), paint),
            Some(SceneNode {
                visual:
                    Visual::Text {
                        text: content,
                        style,
                    },
                ..
            }) => {
                text.draw(canvas, content, style, x, y);
            }
            Some(SceneNode {
                visual: Visual::Icon { icon, size, color },
                ..
            }) => {
                let top = y + (height - size) / 2.0;
                match icon {
                    Icon::Cloud => draw_cloud(canvas, x, top, *size, *color),
                    Icon::Gear => draw_gear(canvas, x, top, *size, *color),
                }
            }
            Some(SceneNode {
                visual: Visual::Group,
                ..
            })
            | None => {}
        }
        for child in self.tree.children(root)? {
            self.paint(child, canvas, text, x, y)?;
        }
        Ok(())
    }
}
