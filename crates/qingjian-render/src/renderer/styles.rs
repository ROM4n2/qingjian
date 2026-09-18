//! 建树用的 Taffy 样式简写。长度都是像素。

use taffy::{
    AlignItems, Dimension, Display, FlexDirection, LengthPercentage, LengthPercentageAuto,
    Position, Rect, Size, Style,
};

/// 横向排子节点，顶部对齐。
pub(super) fn row() -> Style {
    Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: Some(AlignItems::START),
        ..Style::default()
    }
}

/// 纵向排子节点，左对齐。
pub(super) fn column() -> Style {
    Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: Some(AlignItems::START),
        ..Style::default()
    }
}

/// 固定宽高的叶子。
pub(super) fn fixed(width: f32, height: f32) -> Style {
    Style {
        size: Size {
            width: Dimension::length(width),
            height: Dimension::length(height),
        },
        ..Style::default()
    }
}

/// 绝对定位：四边相对父节点的距离，`None` 为不约束。
pub(super) fn absolute(
    left: Option<f32>,
    right: Option<f32>,
    top: Option<f32>,
    bottom: Option<f32>,
) -> Style {
    let edge = |value: Option<f32>| {
        value.map_or(LengthPercentageAuto::auto(), LengthPercentageAuto::length)
    };
    Style {
        position: Position::Absolute,
        inset: Rect {
            left: edge(left),
            right: edge(right),
            top: edge(top),
            bottom: edge(bottom),
        },
        ..Style::default()
    }
}

/// 四边外边距。
pub(super) fn margin(top: f32, right: f32, bottom: f32, left: f32) -> Rect<LengthPercentageAuto> {
    Rect {
        left: LengthPercentageAuto::length(left),
        right: LengthPercentageAuto::length(right),
        top: LengthPercentageAuto::length(top),
        bottom: LengthPercentageAuto::length(bottom),
    }
}

/// 四边内边距。
pub(super) fn padding(top: f32, right: f32, bottom: f32, left: f32) -> Rect<LengthPercentage> {
    Rect {
        left: LengthPercentage::length(left),
        right: LengthPercentage::length(right),
        top: LengthPercentage::length(top),
        bottom: LengthPercentage::length(bottom),
    }
}

/// 只有上外边距。
pub(super) fn margin_top(top: f32) -> Rect<LengthPercentageAuto> {
    margin(top, 0.0, 0.0, 0.0)
}
