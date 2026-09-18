//! 场景图：一棵 Taffy 布局树，每个节点带一个 [`Visual`]。建树 → 布局（文字按实际整形量宽）→ 按树序画。
//!
//! 节点的位置全由布局给出：流内节点按 flex / grid 排，高亮条、光标这类叠在别的节点上的用绝对定位。
//! 单位是像素（主题的点数在建树时已乘倍数），布局不取整，保证与直接算坐标的结果一致。

mod icon;
mod paint;
mod visual;

use std::collections::HashMap;

use taffy::prelude::TaffyMaxContent;
use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree};

use crate::error::RenderError;
use crate::text::{TextPainter, TextSize};

pub(crate) use icon::Icon;
pub(crate) use visual::Visual;

pub(crate) struct Scene {
    /// 布局树，节点上下文是它画什么。
    tree: TaffyTree<Visual>,
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
        self.tree.set_node_context(node, Some(visual))?;
        Ok(node)
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
                        Some(Visual::Text {
                            text: content,
                            style,
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
