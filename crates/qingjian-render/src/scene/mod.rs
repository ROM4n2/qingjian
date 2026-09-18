//! 场景图：一棵 Taffy 布局树，每个节点带一个 [`Visual`]。建树 → 布局（文字按实际整形量宽）→ 按树序画。
//!
//! 节点的位置全由布局给出：流内节点按 flex / grid 排，高亮条、光标这类叠在别的节点上的用绝对定位。
//! 单位是像素（主题的点数在建树时已乘倍数），布局不取整，保证与直接算坐标的结果一致。

mod box_paint;
mod draw_box;
mod draw_effect;
mod draw_visual;
mod effect;
mod extent;
mod fill;
mod icon;
mod node;
mod paint;
mod visual;

use std::collections::HashMap;

use taffy::prelude::TaffyMaxContent;
use taffy::{AlignItems, AvailableSpace, GridPlacement, Line, NodeId, Size, Style, TaffyTree};

use crate::error::RenderError;
use crate::text::{TextPainter, TextSize};

pub(crate) use box_paint::BoxPaint;
pub(crate) use effect::{Effect, EffectKind};
pub(crate) use fill::Fill;
pub(crate) use icon::Icon;
pub(crate) use node::SceneNode;
pub(crate) use visual::Visual;

pub(crate) struct Scene {
    /// 布局树，节点上下文是它画什么。
    tree: TaffyTree<SceneNode>,
}

impl Scene {
    pub(crate) fn new() -> Self {
        let mut tree = TaffyTree::new();
        tree.disable_rounding();
        Self { tree }
    }

    /// 加一个节点，`children` 按画的先后排。
    pub(crate) fn node(
        &mut self,
        style: Style,
        visual: Visual,
        children: &[NodeId],
    ) -> Result<NodeId, RenderError> {
        let node = self.tree.new_with_children(style, children)?;
        self.tree.set_node_context(
            node,
            Some(SceneNode {
                visual,
                opacity: 1.0,
                effects: Vec::new(),
            }),
        )?;
        Ok(node)
    }

    /// 节点连同子节点的不透明度。
    pub(crate) fn set_opacity(&mut self, node: NodeId, opacity: f32) {
        if let Some(context) = self.tree.get_node_context_mut(node) {
            context.opacity = opacity.clamp(0.0, 1.0);
        }
    }

    /// 节点的投影、内阴影。
    pub(crate) fn set_effects(&mut self, node: NodeId, effects: Vec<Effect>) {
        if let Some(context) = self.tree.get_node_context_mut(node) {
            context.effects = effects;
        }
    }

    /// 把表格里的格子放到指定的行、列；`stretch` 时撑满所占的格子（横跨整行的高亮条）。
    pub(crate) fn place(
        &mut self,
        node: NodeId,
        row: Line<GridPlacement>,
        column: Line<GridPlacement>,
        stretch: bool,
    ) -> Result<(), RenderError> {
        let mut style = self.tree.style(node)?.clone();
        style.grid_row = row;
        style.grid_column = column;
        if stretch {
            style.justify_self = Some(AlignItems::STRETCH);
            style.align_self = Some(AlignItems::STRETCH);
        }
        self.tree.set_style(node, style)?;
        Ok(())
    }

    /// 以 `root` 为根按内容撑开算布局，返回根节点的宽高。
    pub(crate) fn layout(
        &mut self,
        root: NodeId,
        text: &mut TextPainter,
    ) -> Result<(f32, f32), RenderError> {
        // 布局会对同一个叶子按不同约束量好几次；单行文字的尺寸与约束无关，量一次记下来
        let mut measured: HashMap<NodeId, TextSize> = HashMap::new();
        self.tree.compute_layout_with_measure(
            root,
            Size::MAX_CONTENT,
            |inputs, node, visual, style| {
                taffy::compute_leaf_layout(
                    inputs,
                    style,
                    |_, _| 0.0,
                    |known, _available: Size<AvailableSpace>| match visual {
                        Some(SceneNode {
                            visual:
                                Visual::Text {
                                    text: content,
                                    style,
                                },
                            ..
                        }) => {
                            let size = *measured
                                .entry(node)
                                .or_insert_with(|| text.measure(content, style));
                            Size {
                                width: known.width.unwrap_or(size.width),
                                height: known.height.unwrap_or(size.height),
                            }
                        }
                        _ => Size {
                            width: known.width.unwrap_or(0.0),
                            height: known.height.unwrap_or(0.0),
                        },
                    },
                )
            },
        )?;
        let size = self.tree.layout(root)?.size;
        Ok((size.width, size.height))
    }
}
