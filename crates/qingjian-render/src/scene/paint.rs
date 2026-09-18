//! 按树序画：先画节点自己（投影垫底、内阴影压在填充上；自己不画的容器取子节点当阴影形状），再画子节点；坐标是父节点位置加布局给的相对位置。
//! 不透明度小于 1 的节点连同子树先画到离屏图层，再按不透明度合成。

use taffy::NodeId;

use super::cache_key::{CacheKey, Slot};
use super::draw_effect::effect_mask;
use super::draw_visual::{draw_shape, draw_visual};
use super::{CachedVisual, Fill};
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
        let opacity = self.tree.get_node_context(root).map_or(1.0, |node| {
            node.placed.map_or(node.opacity, |placed| placed.opacity)
        });
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
        let placed = self
            .tree
            .get_node_context(root)
            .and_then(|node| node.placed);
        // 动画中的节点按插值后的位置画，子树跟着走
        let rect = match placed {
            Some(placed) => (placed.x, placed.y, placed.width, placed.height),
            None => (
                x + layout.location.x,
                y + layout.location.y,
                layout.size.width,
                layout.size.height,
            ),
        };
        let (x, y) = (rect.0, rect.1);
        if let Some(SceneNode {
            visual, effects, ..
        }) = self.tree.get_node_context(root)
        {
            let children = self.tree.children(root)?;
            for kind in [EffectKind::DropShadow, EffectKind::InnerShadow] {
                if kind == EffectKind::InnerShadow {
                    self.draw_own(canvas, text, root, visual, rect)?;
                }
                for (index, effect) in effects.iter().enumerate() {
                    if effect.kind != kind {
                        continue;
                    }
                    let key = CacheKey::new(root, Slot::Effect(index), rect, canvas);
                    let cached = self.effect_cache.borrow().get(&key).cloned();
                    let mask = match cached {
                        Some(mask) => mask,
                        None => {
                            // 自己不画东西的容器（译文、拼音行）拿子节点当形状
                            let mut shape = |layer: &mut Canvas, (x, y, width, height), spread| {
                                match visual {
                                    Visual::Group => {
                                        for &child in &children {
                                            self.paint(child, layer, text, x, y)?;
                                        }
                                    }
                                    _ => draw_shape(
                                        layer,
                                        text,
                                        visual,
                                        (x, y, width, height),
                                        spread,
                                    ),
                                }
                                Ok(())
                            };
                            let bounds = (canvas.width(), canvas.height());
                            let mask = effect_mask(bounds, rect, effect, &mut shape)?;
                            self.effect_cache.borrow_mut().insert(key, mask.clone());
                            mask
                        }
                    };
                    if let Some(mask) = mask {
                        canvas.blend_mask(
                            mask.left,
                            mask.top,
                            mask.width,
                            mask.height,
                            &mask.alpha,
                            effect.color,
                        );
                    }
                }
            }
        }
        for child in self.tree.children(root)? {
            self.paint(child, canvas, text, x, y)?;
        }
        Ok(())
    }

    /// 画节点自己的画面。图片填充的框先画进一块离屏图、按键缓存，动画帧里没动就直接贴回。
    fn draw_own(
        &self,
        canvas: &mut Canvas,
        text: &mut TextPainter,
        node: NodeId,
        visual: &Visual,
        rect: (f32, f32, f32, f32),
    ) -> Result<(), RenderError> {
        let is_image =
            matches!(visual, Visual::Box(paint) if matches!(paint.fill, Some(Fill::Image { .. })));
        if !is_image {
            draw_visual(canvas, text, visual, rect);
            return Ok(());
        }
        let key = CacheKey::new(node, Slot::Visual, rect, canvas);
        if !self.visual_cache.borrow().contains_key(&key) {
            let (x, y, width, height) = rect;
            let (left, top) = (x.floor(), y.floor());
            let size = (
                ((x + width).ceil() - left).max(1.0) as u32,
                ((y + height).ceil() - top).max(1.0) as u32,
            );
            let mut layer = Canvas::new(size.0, size.1)?;
            draw_visual(&mut layer, text, visual, (x - left, y - top, width, height));
            self.visual_cache.borrow_mut().insert(
                key,
                CachedVisual {
                    left: left as i32,
                    top: top as i32,
                    pixmap: layer.into_pixmap(),
                },
            );
        }
        if let Some(cached) = self.visual_cache.borrow().get(&key) {
            canvas.composite(cached.left, cached.top, &cached.pixmap);
        }
        Ok(())
    }
}
