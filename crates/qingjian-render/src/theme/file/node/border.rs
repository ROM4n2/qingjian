//! 框的边框：画在盒子内侧，不占布局空间。

use serde::Deserialize;

use crate::theme::file::ColorSpec;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct BorderSpec {
    /// 宽度（点）。
    pub(crate) width: f32,

    pub(crate) color: ColorSpec,
}
