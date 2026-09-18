//! 主题：一份 `theme.json`（图层树 + 组件 + 颜色变量 + 文字样式）加当前外观（浅色 / 深色）。格式见 `docs/design/theme.md`。
//!
//! 内置主题「青简」随 crate 编进来（`themes/qingjian/theme.json`），只解析一次；用户主题用 [`Theme::from_json`]。
//! 渲染时引用不到的颜色、样式退回缺省值，加载时 [`validate`] 先把这类问题记成警告。

mod error;
pub(crate) mod file;
mod font_spec;
mod validate;

use std::sync::{Arc, OnceLock};

use crate::color::Color;

pub use error::ThemeError;
pub use font_spec::FontSpec;

use file::{ColorRef, SCHEMA, ThemeFile};

/// 内置主题的源文件。
const BUILTIN: &str = include_str!("../../themes/qingjian/theme.json");

/// 引用不到的文字样式退回这个（点）。
const FALLBACK_FONT: FontSpec = FontSpec::new(16.0, 19.0);

#[derive(Debug, Clone)]
pub struct Theme {
    /// 解析后的主题文件，同一主题的浅色 / 深色共用。
    file: Arc<ThemeFile>,

    /// 用深色那一套值。
    dark: bool,
}

impl Theme {
    /// 内置主题，浅色。
    pub fn light() -> Self {
        Self::builtin(false)
    }

    /// 内置主题，深色。
    pub fn dark() -> Self {
        Self::builtin(true)
    }

    /// 从 `theme.json` 的内容读主题。引用不到的名字只记警告。
    pub fn from_json(json: &str, dark: bool) -> Result<Self, ThemeError> {
        let file: ThemeFile = serde_json::from_str(json)?;
        if file.schema > SCHEMA {
            tracing::warn!(
                id = file.meta.id,
                schema = file.schema,
                supported = SCHEMA,
                "主题格式比当前版本新，只按认得的部分画"
            );
        }
        for problem in validate::problems(&file) {
            tracing::warn!(id = file.meta.id, "主题引用有误：{problem}");
        }
        Ok(Self {
            file: Arc::new(file),
            dark,
        })
    }

    /// 同一主题换外观。
    pub fn with_dark(&self, dark: bool) -> Self {
        Self {
            file: Arc::clone(&self.file),
            dark,
        }
    }

    /// 主题 id。
    pub fn id(&self) -> &str {
        &self.file.meta.id
    }

    /// 显示名。
    pub fn name(&self) -> &str {
        &self.file.meta.name
    }

    pub fn author(&self) -> &str {
        &self.file.meta.author
    }

    /// SPDX 许可证标识。
    pub fn license(&self) -> &str {
        &self.file.meta.license
    }

    fn builtin(dark: bool) -> Self {
        static FILE: OnceLock<Theme> = OnceLock::new();
        FILE.get_or_init(|| {
            Self::from_json(BUILTIN, false).expect("内置主题 themes/qingjian/theme.json 解析失败")
        })
        .with_dark(dark)
    }

    pub(crate) fn file(&self) -> &ThemeFile {
        &self.file
    }

    /// 颜色引用按当前外观取值；变量不存在时透明。
    pub(crate) fn color(&self, color: &ColorRef) -> Color {
        match color {
            ColorRef::Literal(color) => *color,
            ColorRef::Variable(name) => self
                .file
                .variables
                .get(name)
                .map_or(Color::rgba(0, 0, 0, 0), |value| value.get(self.dark).0),
        }
    }

    /// 命名文字样式（点）。
    pub(crate) fn font(&self, name: &str) -> FontSpec {
        self.file
            .text
            .styles
            .get(name)
            .copied()
            .unwrap_or(FALLBACK_FONT)
    }

    /// 当前外观下的文字覆盖率 gamma。
    pub(crate) fn text_gamma(&self) -> f32 {
        self.file.text.gamma.get(self.dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_theme_parses_without_problems() {
        let theme = Theme::light();
        assert_eq!(theme.id(), "qingjian");
        assert!(validate::problems(theme.file()).is_empty());
        assert_eq!(
            theme.color(&ColorRef::Variable("accent".to_owned())),
            Color::rgba(176, 206, 125, 127)
        );
        assert_eq!(
            Theme::dark().color(&ColorRef::Variable("accent".to_owned())),
            Color::rgb(36, 76, 36)
        );
    }
}
