//! 加载时检查一遍引用（颜色变量、文字样式、组件），有问题记警告；渲染时引用不到的退回缺省值，不让整个主题失败。

use super::file::node::{NodeKind, NodeSpec};
use super::file::{ColorRef, ThemeFile};

/// 列出主题里引用不到的名字。
pub(super) fn problems(file: &ThemeFile) -> Vec<String> {
    let mut found = Vec::new();
    let mut check = Checker {
        file,
        found: &mut found,
    };
    for (name, component) in &file.components {
        check.node(component, &format!("components.{name}"));
    }
    check.node(&file.windows.vertical, "windows.vertical");
    check.node(&file.windows.horizontal, "windows.horizontal");
    let status = &file.status;
    check.font(&status.font, "status");
    for color in [
        &status.background,
        &status.separator,
        &status.normal,
        &status.emphasized,
        &status.gear,
    ] {
        check.color(color, "status");
    }
    found
}

struct Checker<'a> {
    file: &'a ThemeFile,

    found: &'a mut Vec<String>,
}

impl Checker<'_> {
    fn node(&mut self, node: &NodeSpec, path: &str) {
        match &node.kind {
            NodeKind::Frame { fill, children, .. } => {
                if let Some(fill) = fill {
                    self.color(fill, path);
                }
                for (i, child) in children.iter().enumerate() {
                    self.node(child, &format!("{path}.children[{i}]"));
                }
            }
            NodeKind::Text { font, color, .. } => {
                self.font(font, path);
                self.color(color, path);
            }
            NodeKind::Icon { color, .. } => self.color(color, path),
            NodeKind::Preedit {
                font,
                typed,
                rest,
                struck,
                caret,
            } => {
                self.font(font, path);
                for color in [typed, rest, struck, &caret.color] {
                    self.color(color, path);
                }
            }
            NodeKind::Annotation {
                font,
                gloss,
                fresh,
                faint,
                ..
            } => {
                self.font(font, path);
                for color in [gloss, fresh, faint] {
                    self.color(color, path);
                }
            }
            NodeKind::Use { component } | NodeKind::Repeat { component, .. } => {
                if !self.file.components.contains_key(component) {
                    self.found
                        .push(format!("{path}: 组件 {component:?} 不存在"));
                }
            }
        }
    }

    fn color(&mut self, color: &ColorRef, path: &str) {
        if let ColorRef::Variable(name) = color
            && !self.file.variables.contains_key(name)
        {
            self.found.push(format!("{path}: 颜色变量 @{name} 不存在"));
        }
    }

    fn font(&mut self, font: &str, path: &str) {
        if !self.file.text.styles.contains_key(font) {
            self.found.push(format!("{path}: 文字样式 {font:?} 不存在"));
        }
    }
}
