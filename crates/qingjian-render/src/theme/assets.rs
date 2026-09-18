//! 主题的图片素材：加载主题时从主题目录读进来、解码一次，渲染时按路径取。内置主题不带图片。
//!
//! 路径相对 `theme.json`，不能是绝对路径、不能含 `..`（主题不能读主题目录以外的文件）；只认 PNG，边长上限 [`MAX_SIDE`]。
//! 读不进来的记警告，用到它的填充不画。

use std::collections::HashMap;
use std::path::{Component, Path};
use std::sync::Arc;

use tiny_skia::Pixmap;

use super::file::ThemeFile;
use super::file::node::{FillSpec, NodeKind, NodeSpec};

/// 图片边长上限（像素）。
const MAX_SIDE: u32 = 4096;

#[derive(Debug, Default)]
pub(crate) struct Assets {
    /// 主题里写的路径 → 解码好的预乘位图。
    images: HashMap<String, Arc<Pixmap>>,
}

impl Assets {
    /// 读 `file` 里用到的全部图片；`dir` 是主题目录。
    pub(crate) fn load(file: &ThemeFile, dir: &Path) -> Self {
        let mut paths = Vec::new();
        for component in file.components.values() {
            collect(component, &mut paths);
        }
        collect(&file.windows.vertical, &mut paths);
        collect(&file.windows.horizontal, &mut paths);
        let mut images = HashMap::new();
        for path in paths {
            if images.contains_key(&path) {
                continue;
            }
            match load_image(dir, &path) {
                Ok(image) => {
                    images.insert(path, Arc::new(image));
                }
                Err(problem) => {
                    tracing::warn!(id = file.meta.id, path, "主题图片{problem}，不画");
                }
            }
        }
        Self { images }
    }

    pub(crate) fn image(&self, path: &str) -> Option<Arc<Pixmap>> {
        self.images.get(path).cloned()
    }
}

fn load_image(dir: &Path, path: &str) -> Result<Pixmap, &'static str> {
    let relative = Path::new(path);
    if !relative
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err("路径不在主题目录里");
    }
    let image = Pixmap::load_png(dir.join(relative)).map_err(|_| "读不进来（只认 PNG）")?;
    if image.width() > MAX_SIDE || image.height() > MAX_SIDE {
        return Err("太大");
    }
    Ok(image)
}

/// 收集节点树里的图片路径。
fn collect(node: &NodeSpec, out: &mut Vec<String>) {
    if let NodeKind::Frame { fill, children, .. } = &node.kind {
        if let Some(FillSpec::Image { image, .. }) = fill {
            out.push(image.clone());
        }
        for child in children {
            collect(child, out);
        }
    }
}
