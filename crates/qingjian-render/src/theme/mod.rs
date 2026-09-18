//! 主题：一份 `theme.json`（图层树 + 组件 + 颜色变量 + 文字样式）加当前外观（浅色 / 深色）。格式见 `docs/design/theme.md`。
//!
//! 内置主题随 crate 编进来（`themes/<id>/theme.json`），第一次用到时解析一次；用户主题用 [`Theme::from_json`]。
//! 主题可以 `"extends": "<内置主题 id>"`，只写要改的部分（见 `extends.rs`）。
//! 渲染时引用不到的颜色、样式退回缺省值，加载时 [`validate`] 先把这类问题记成警告。

mod assets;
mod error;
mod extends;
pub(crate) mod file;
mod font_spec;
mod font_weight;
mod library;
mod validate;

use std::path::Path;
use std::sync::{Arc, OnceLock};

use tiny_skia::Pixmap;

use crate::color::Color;

pub use error::ThemeError;
pub use font_spec::FontSpec;
pub use font_weight::FontWeight;
pub use library::ThemeLibrary;

use assets::Assets;
use file::{ColorRef, FontRef, LockedAppearance, SCHEMA, ThemeFile};

/// 内置主题：id 与源文件，按设置界面列出的顺序；第一个是缺省主题。
const BUILTINS: [(&str, &str); 3] = [
    ("qingjian", include_str!("../../themes/qingjian/theme.json")),
    (
        "system-blue",
        include_str!("../../themes/system-blue/theme.json"),
    ),
    ("wechat", include_str!("../../themes/wechat/theme.json")),
];

/// 引用不到的文字样式退回这个（点）。
const FALLBACK_FONT: FontSpec = FontSpec::new(16.0, 19.0);

#[derive(Debug, Clone)]
pub struct Theme {
    /// 解析后的主题文件，同一主题的浅色 / 深色共用。
    file: Arc<ThemeFile>,

    /// 用深色那一套值。
    dark: bool,

    /// 图片素材（主题目录里的），同一主题的浅色 / 深色共用。
    assets: Arc<Assets>,
}

impl Theme {
    /// 缺省内置主题，浅色。
    pub fn light() -> Self {
        Self::default_builtin().clone()
    }

    /// 缺省内置主题，深色。
    pub fn dark() -> Self {
        Self::default_builtin().with_dark(true)
    }

    /// 按 id 找内置主题；没有这个 id 时为 `None`。
    pub fn builtin(id: &str, dark: bool) -> Option<Self> {
        builtins()
            .iter()
            .find(|theme| theme.id() == id)
            .map(|theme| theme.with_dark(dark))
    }

    /// 全部内置主题（浅色），按设置界面列出的顺序。
    pub fn builtins() -> &'static [Theme] {
        builtins()
    }

    /// 从 `theme.json` 的内容读主题。可以 `extends` 内置主题；引用不到的名字只记警告。
    pub fn from_json(json: &str, dark: bool) -> Result<Self, ThemeError> {
        let value = extends::resolve(json, &builtin_source)?;
        let file: ThemeFile = serde_json::from_value(value)?;
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
        let theme = Self {
            file: Arc::new(file),
            dark,
            assets: Arc::default(),
        };
        Ok(theme.with_dark(dark))
    }

    /// 读主题目录：`dir/theme.json` 加它用到的图片（路径相对 `dir`）。
    pub fn from_dir(dir: &Path, dark: bool) -> Result<Self, ThemeError> {
        let json = std::fs::read_to_string(dir.join("theme.json"))?;
        let mut theme = Self::from_json(&json, dark)?;
        theme.assets = Arc::new(Assets::load(&theme.file, dir));
        Ok(theme)
    }

    /// 同一主题换外观；主题锁定了外观（`meta.appearance`）时不换。
    pub fn with_dark(&self, dark: bool) -> Self {
        Self {
            file: Arc::clone(&self.file),
            dark: self.locked_dark().unwrap_or(dark),
            assets: Arc::clone(&self.assets),
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

    fn default_builtin() -> &'static Theme {
        &builtins()[0]
    }

    /// 锁定的外观：`Some(true)` 只有深色，`Some(false)` 只有浅色。
    fn locked_dark(&self) -> Option<bool> {
        self.file
            .meta
            .appearance
            .map(|appearance| appearance == LockedAppearance::Dark)
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

    /// 主题里写的图片路径对应的位图；没有或没读进来为 `None`。
    pub(crate) fn image(&self, path: &str) -> Option<Arc<Pixmap>> {
        self.assets.image(path)
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

    /// 节点里的 `font`：样式名，或在样式上改字号 / 字重。
    pub(crate) fn font_ref(&self, font: &FontRef) -> FontSpec {
        font.apply(font.base().map_or(FALLBACK_FONT, |name| self.font(name)))
    }

    /// 当前外观下的文字覆盖率 gamma。
    pub(crate) fn text_gamma(&self) -> f32 {
        self.file.text.gamma.get(self.dark)
    }
}

/// 解析好的内置主题，第一次用到时解析。内置主题是随包的，解析失败是 bug。
fn builtins() -> &'static [Theme] {
    static THEMES: OnceLock<Vec<Theme>> = OnceLock::new();
    THEMES.get_or_init(|| {
        BUILTINS
            .iter()
            .map(|(id, source)| {
                let theme = Theme::from_json(source, false)
                    .unwrap_or_else(|error| panic!("内置主题 {id} 解析失败：{error}"));
                debug_assert_eq!(theme.id(), *id, "内置主题 id 与目录名不一致");
                theme
            })
            .collect()
    })
}

/// `extends` 按 id 找内置主题的源文件。
fn builtin_source(id: &str) -> Option<&'static str> {
    BUILTINS
        .iter()
        .find(|(builtin, _)| *builtin == id)
        .map(|(_, source)| *source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_themes_parse_without_problems() {
        for theme in Theme::builtins() {
            assert!(
                validate::problems(theme.file()).is_empty(),
                "{}: {:?}",
                theme.id(),
                validate::problems(theme.file())
            );
        }
        assert_eq!(Theme::builtins().len(), BUILTINS.len());
        let theme = Theme::light();
        assert_eq!(theme.id(), "qingjian");
        assert!(Theme::builtin("wechat", true).is_some());
        assert!(Theme::builtin("nope", false).is_none());
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
