//! 场景树节点的上下文：画什么、效果、整棵子树的不透明度，以及过渡用的配对名与动画中的位置。

use super::effect::Effect;
use super::visual::Visual;
use crate::animation::{Easing, Placement};

#[derive(Debug, Clone)]
pub(crate) struct SceneNode {
    pub(crate) visual: Visual,

    /// 不透明度（0–1），小于 1 时子树先画到离屏图层再合成。
    pub(crate) opacity: f32,

    /// 投影、内阴影，按写的顺序画。
    pub(crate) effects: Vec<Effect>,

    /// 过渡配对名（主题里的 `id`，重复时带序号）与过渡参数；没写 `transition` 的为 `None`。
    pub(crate) transition: Option<(String, std::time::Duration, Easing)>,

    /// 动画中的位置（画布像素）与不透明度，盖过布局算的；不在动画中为 `None`。
    pub(crate) placed: Option<Placement>,
}
