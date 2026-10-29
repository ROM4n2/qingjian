//! 「高级」页的设置导入导出：只搬设置，短语、云服务密钥与状态条位置留在这台电脑（规则在
//! `qingjian_platform::export_settings` / `import_settings`）。导入前把现在的配置另存一份。

use qingjian_platform::{export_settings, import_settings};

use super::Settings;

/// 消息挂在哪一行下面。
const SLOT: &str = "config-transfer";

/// 导入前现在的配置另存成这个名字（与配置同目录），导错了能拷回来。
const BEFORE_IMPORT: &str = "config.before-import.toml";

impl Settings {
    pub(super) fn export_config(&mut self) {
        let source = match std::fs::read_to_string(&self.path) {
            Ok(source) => source,
            Err(error) => {
                self.notice = Some((SLOT, format!("读不了配置文件：{error}")));
                return;
            }
        };
        let exported = match export_settings(&source) {
            Ok(exported) => exported,
            Err(error) => {
                self.notice = Some((SLOT, error));
                return;
            }
        };
        let name = format!("青简设置-{}.toml", jiff::Zoned::now().date());
        let Some(target) = rfd::FileDialog::new()
            .set_title("导出设置")
            .set_file_name(&name)
            .add_filter("青简设置", &["toml"])
            .save_file()
        else {
            return;
        };
        let notice = match std::fs::write(&target, exported) {
            Ok(()) => format!("已导出到 {}", target.display()),
            Err(error) => format!("写不进去：{error}"),
        };
        self.notice = Some((SLOT, notice));
    }

    pub(super) fn import_config(&mut self) {
        let Some(source) = rfd::FileDialog::new()
            .set_title("导入设置")
            .add_filter("青简设置", &["toml"])
            .add_filter("所有文件", &["*"])
            .pick_file()
        else {
            return;
        };
        let current = std::fs::read_to_string(&self.path).unwrap_or_default();
        let result = std::fs::read_to_string(&source)
            .map_err(|error| format!("读不了这个文件：{error}"))
            .and_then(|imported| import_settings(&current, &imported))
            .and_then(|merged| {
                let backup = self.path.with_file_name(BEFORE_IMPORT);
                if !current.is_empty() {
                    std::fs::write(&backup, &current)
                        .map_err(|error| format!("没能先备份现在的配置，没有导入：{error}"))?;
                }
                std::fs::write(&self.path, merged).map_err(|error| format!("写不进去：{error}"))?;
                Ok(format!("已导入；原来的配置存成了同目录的 {BEFORE_IMPORT}"))
            });
        self.reload();
        self.notice = Some((SLOT, result.unwrap_or_else(|error| error)));
    }
}
