//! 主题文件读不进来。

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    /// 不是合法的 JSON，或结构对不上（缺必填项、类型错）。
    #[error("invalid theme file: {0}")]
    Json(#[from] serde_json::Error),
}
