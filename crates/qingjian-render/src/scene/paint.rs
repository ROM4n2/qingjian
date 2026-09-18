//! 按树序画：先画节点自己，再画子节点；坐标是父节点位置加布局给的相对位置。

use taffy::NodeId;

use super::{Icon, Scene, Visual};
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
        let layout = self.tree.layout(root)?;
        let (x, y) = (x + layout.location.x, y + layout.location.y);
        let (width, height) = (layout.size.width, layout.size.height);
        match self.tree.get_node_context(root) {
            Some(Visual::Fill { color, radius }) if *radius > 0.0 => {
                canvas.fill_round_rect(x, y, width, height, *radius, *color);
            }
            Some(Visual::Fill { color, .. }) => canvas.fill_rect(x, y, width, height, *color),
            Some(Visual::Text {
                text: content,
                style,
            }) => {
                text.draw(canvas, content, style, x, y);
            }
            Some(Visual::Icon { icon, size, color }) => {
                let top = y + (height - size) / 2.0;
                match icon {
                    Icon::Cloud => draw_cloud(canvas, x, top, *size, *color),
                    Icon::Gear => draw_gear(canvas, x, top, *size, *color),
                }
            }
            Some(Visual::Group) | None => {}
        }
        for child in self.tree.children(root)? {
            self.paint(child, canvas, text, x, y)?;
        }
        Ok(())
    }
}
