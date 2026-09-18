//! 拼音行里的光标。

use serde::Deserialize;

use crate::theme::file::ColorRef;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct CaretSpec {
    /// 宽度（点）。
    pub(crate) width: f32,

    pub(crate) color: ColorRef,
}
