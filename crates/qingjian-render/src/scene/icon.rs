//! 渲染器自己画的矢量小图标。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Icon {
    /// 云联想的云朵。
    Cloud,

    /// 状态条打开设置的齿轮。
    Gear,
}
