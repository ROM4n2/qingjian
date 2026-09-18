//! 主题的元数据。

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Meta {
    /// 主题 id，也是主题目录名。
    pub(crate) id: String,

    /// 显示名。
    pub(crate) name: String,

    #[serde(default)]
    pub(crate) author: String,

    /// SPDX 许可证标识。
    #[serde(default)]
    pub(crate) license: String,
}
