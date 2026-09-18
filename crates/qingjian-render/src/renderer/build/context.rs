//! 实例化时的数据上下文：整帧，加上 `repeat` 展开中的那一项。显示条件（`when`）与绑定（`bind`）都在这里求值。
//!
//! 名字先在候选项里找，再在整帧里找；不认识的条件为假、不认识的绑定为空（节点不画）。

use super::row::RowContext;
use crate::frame::{Frame, Row, Tone};

#[derive(Debug, Clone, Copy)]
pub(super) struct Context<'a> {
    pub(super) frame: &'a Frame,

    /// 在 `repeat` 里时绑定的那一项。
    pub(super) row: Option<RowContext<'a>>,
}

impl<'a> Context<'a> {
    pub(super) fn new(frame: &'a Frame) -> Self {
        Self { frame, row: None }
    }

    /// 展开列表时每一项的上下文。
    pub(super) fn with_row(self, row: &'a Row, index: usize, count: usize) -> Self {
        Self {
            row: Some(RowContext { row, index, count }),
            ..self
        }
    }

    /// `when` 求值：`a|b` 任一成立，`!a` 取反。
    pub(super) fn holds(&self, condition: &str) -> bool {
        condition.split('|').any(|term| {
            let term = term.trim();
            match term.strip_prefix('!') {
                Some(name) => !self.flag(name),
                None => self.flag(term),
            }
        })
    }

    fn flag(&self, name: &str) -> bool {
        if let Some(RowContext { row, index, count }) = self.row {
            match name {
                "highlighted" => return self.frame.highlighted == Some(index),
                "cloud" => return row.cloud,
                "annotation" => return !row.annotation.is_empty(),
                "first" => return index == 0,
                "last" => return index + 1 == count,
                _ => {}
            }
        }
        let frame = self.frame;
        match name {
            "preedit" => frame.preedit.is_some(),
            "trailing" => frame.trailing().is_some(),
            "trailing.cloud" => frame.trailing().is_some_and(|(_, cloud)| cloud),
            "page" => frame.footer.is_some(),
            "candidates" => !frame.rows.is_empty(),
            "annotations" => frame.rows.iter().any(|row| !row.annotation.is_empty()),
            "highlighted" => self.highlighted().is_some(),
            "highlighted.annotation" => self
                .highlighted()
                .is_some_and(|row| !row.annotation.is_empty()),
            _ => false,
        }
    }

    /// 文字绑定。
    pub(super) fn text(&self, name: &str) -> Option<&'a str> {
        if let Some(RowContext { row, .. }) = self.row {
            match name {
                "index" => return Some(&row.index),
                "text" => return Some(&row.text),
                _ => {}
            }
        }
        match name {
            "page" => self.frame.footer.as_deref(),
            "trailing.text" => self.frame.trailing().map(|(text, _)| text),
            _ => None,
        }
    }

    /// 译文片段绑定。
    pub(super) fn annotation(&self, name: &str) -> Option<&'a [(String, Tone)]> {
        match (name, self.row) {
            ("annotation", Some(RowContext { row, .. })) => Some(&row.annotation),
            ("highlighted.annotation", _) => {
                self.highlighted().map(|row| row.annotation.as_slice())
            }
            _ => None,
        }
    }

    /// 列表绑定。
    pub(super) fn list(&self, name: &str) -> Option<&'a [Row]> {
        (name == "candidates").then_some(self.frame.rows.as_slice())
    }

    fn highlighted(&self) -> Option<&'a Row> {
        self.frame.highlighted.and_then(|i| self.frame.rows.get(i))
    }
}
