//! 设置里的字号（点）：0 或不是正数为「用主题的」，不限上限。包一层是为了让配置能整体比较（`f32` 不是 `Eq`）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FontSize(pub f32);

impl FontSize {
    /// 填了的字号；没填（0、负数、NaN）为 `None`。
    pub fn get(self) -> Option<f32> {
        (self.0.is_finite() && self.0 > 0.0).then_some(self.0)
    }

    /// 设置界面里填的数；清空、负数、NaN 都是「用主题的」。
    pub fn from_input(size: Option<f64>) -> Self {
        Self(size.map_or(0.0, |size| size as f32))
    }
}

impl From<FontSize> for toml_edit::Value {
    /// 整数写成整数（`16` 而不是 `16.0`），配置文件里好读；没填写 0。
    fn from(size: FontSize) -> Self {
        let size = size.get().map_or(0.0, f64::from);
        if size.fract() == 0.0 && size < i64::MAX as f64 {
            (size as i64).into()
        } else {
            size.into()
        }
    }
}

impl PartialEq for FontSize {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for FontSize {}
