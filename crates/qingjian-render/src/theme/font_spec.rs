//! 一种字体用法：字号、行高（点）与字重。字族不在这里定，由字体库按平台给界面字体。

use serde::Deserialize;

use super::FontWeight;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct FontSpec {
    /// 字号。
    pub size: f32,

    /// 行高：一行文字占的高度，字形在其中垂直居中。
    pub line_height: f32,

    /// 字重，不写为常规。
    #[serde(default)]
    pub weight: FontWeight,
}

impl FontSpec {
    pub const fn new(size: f32, line_height: f32) -> Self {
        Self {
            size,
            line_height,
            weight: FontWeight::REGULAR,
        }
    }

    /// 点 → 像素。
    pub(crate) fn scaled(self, scale: f32) -> Self {
        Self {
            size: self.size * scale,
            line_height: self.line_height * scale,
            weight: self.weight,
        }
    }
}
