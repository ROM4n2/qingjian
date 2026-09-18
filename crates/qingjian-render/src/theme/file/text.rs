//! 文字设置：命名的字号行高（节点用 `"font": "名字"` 引用）与覆盖率 gamma。

use std::collections::HashMap;

use serde::Deserialize;

use super::adaptive::Adaptive;
use crate::theme::FontSpec;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TextSettings {
    /// 文字抗锯齿覆盖率的 gamma：小于 1 笔画显粗，见 `docs/design/rendering.md`。
    pub(crate) gamma: Adaptive<f32>,

    /// 命名的文字样式。
    pub(crate) styles: HashMap<String, FontSpec>,
}
