//! 场景树节点的上下文：画什么、效果，加整棵子树的不透明度。

use super::effect::Effect;
use super::visual::Visual;

#[derive(Debug, Clone)]
pub(crate) struct SceneNode {
    pub(crate) visual: Visual,

    /// 不透明度（0–1），小于 1 时子树先画到离屏图层再合成。
    pub(crate) opacity: f32,

    /// 投影、内阴影，按写的顺序画。
    pub(crate) effects: Vec<Effect>,
}
