//! 一次渲染期间的上下文：主题按倍数换算后的像素值与各处文字样式。

use crate::color::Color;
use crate::frame::Tone;
use crate::text::TextStyle;
use crate::theme::{FontSpec, Theme};

use super::{CLOUD_GAP, CLOUD_SIZE};

pub(super) struct Metrics<'a> {
    pub(super) theme: &'a Theme,

    /// 点 → 像素的倍数。
    pub(super) scale: f32,
}

impl Metrics<'_> {
    pub(super) fn px(&self, points: f32) -> f32 {
        points * self.scale
    }

    pub(super) fn padding(&self) -> f32 {
        self.px(self.theme.padding)
    }

    pub(super) fn row_padding(&self) -> f32 {
        self.px(self.theme.row_padding)
    }

    pub(super) fn column_gap(&self) -> f32 {
        self.px(self.theme.column_gap)
    }

    pub(super) fn corner_radius(&self) -> f32 {
        self.px(self.theme.corner_radius)
    }

    /// 候选词的行高。
    pub(super) fn text_line_height(&self) -> f32 {
        self.px(self.theme.text_font.line_height)
    }

    /// 译文、拼音行的行高。
    pub(super) fn annotation_line_height(&self) -> f32 {
        self.px(self.theme.annotation_font.line_height)
    }

    /// 候选行的高度（上下各留 `row_padding`）。
    pub(super) fn row_height(&self) -> f32 {
        self.text_line_height() + self.row_padding() * 2.0
    }

    pub(super) fn style(&self, font: FontSpec, color: Color) -> TextStyle {
        TextStyle::new(
            font.scaled(self.scale),
            font.size,
            color,
            self.theme.text_gamma,
        )
    }

    pub(super) fn text_style(&self) -> TextStyle {
        self.style(self.theme.text_font, self.theme.colors.text)
    }

    pub(super) fn annotation_style(&self, color: Color) -> TextStyle {
        self.style(self.theme.annotation_font, color)
    }

    pub(super) fn index_style(&self) -> TextStyle {
        self.style(self.theme.index_font, self.theme.colors.index)
    }

    pub(super) fn tone_color(&self, tone: Tone) -> Color {
        match tone {
            Tone::Gloss => self.theme.colors.gloss,
            Tone::Fresh => self.theme.colors.fresh,
            Tone::Faint => self.theme.colors.pos,
        }
    }

    /// 小字（序号、译文）相对候选词往下挪多少，让两者底部对齐。
    pub(super) fn small_offset(&self) -> f32 {
        (self.text_line_height() - self.annotation_line_height()).max(0.0)
    }

    pub(super) fn cloud_size(&self) -> f32 {
        self.px(CLOUD_SIZE)
    }

    pub(super) fn cloud_gap(&self) -> f32 {
        self.px(CLOUD_GAP)
    }
}
