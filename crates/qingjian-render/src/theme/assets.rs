//! 主题的图片与字体素材：加载主题时从主题目录读进来，图片解码一次、渲染时按路径取；字体只核对路径，由渲染器加载。内置主题不带素材。
//!
//! 路径相对 `theme.json`，不能是绝对路径、不能含 `..`（主题不能读主题目录以外的文件）；图片只认 PNG，边长上限 [`MAX_SIDE`]；
//! 字体单个文件上限 [`MAX_FONT_BYTES`]。读不进来的记警告，用到图片的填充不画，用到字体的样式按回退链往后找。

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use tiny_skia::Pixmap;

use super::file::ThemeFile;
use super::file::node::{FillSpec, NodeKind, NodeSpec};

/// 图片边长上限（像素）。
const MAX_SIDE: u32 = 4096;

/// 随主题带的单个字体文件上限（字节）：裁过字的中文字体在 10 MB 以内，整个主题包上限 32 MB。
const MAX_FONT_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Default)]
pub(crate) struct Assets {
    /// 主题里写的路径 → 解码好的预乘位图。
    images: HashMap<String, Arc<Pixmap>>,

    /// 随主题带的字体文件。
    fonts: Vec<PathBuf>,
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
        let mut fonts = Vec::new();
        for font in &file.fonts {
            match font_path(dir, &font.file) {
                Ok(path) if !fonts.contains(&path) => fonts.push(path),
                Ok(_) => {}
                Err(problem) => {
                    tracing::warn!(
                        id = file.meta.id,
                        path = font.file,
                        "主题字体{problem}，不用"
                    );
                }
            }
        }
        Self { images, fonts }
    }

    pub(crate) fn image(&self, path: &str) -> Option<Arc<Pixmap>> {
        self.images.get(path).cloned()
    }

    pub(crate) fn fonts(&self) -> &[PathBuf] {
        &self.fonts
    }
}

/// 主题里写的相对路径 → 主题目录里的路径；绝对路径与 `..` 拒掉。
fn inside(dir: &Path, path: &str) -> Result<PathBuf, &'static str> {
    let relative = Path::new(path);
    if !relative
        .components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err("路径不在主题目录里");
    }
    Ok(dir.join(relative))
}

fn font_path(dir: &Path, path: &str) -> Result<PathBuf, &'static str> {
    let path = inside(dir, path)?;
    let size = std::fs::metadata(&path).map_err(|_| "不存在")?.len();
    if size > MAX_FONT_BYTES {
        return Err("太大");
    }
    Ok(path)
}

fn load_image(dir: &Path, path: &str) -> Result<Pixmap, &'static str> {
    let image = Pixmap::load_png(inside(dir, path)?).map_err(|_| "读不进来（只认 PNG）")?;
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
