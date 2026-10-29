//! 「高级」页：配置文件与数据目录、设置的导入导出，日志，输入日志。

use bamboo_slip::taffy::{FlexDirection, NodeId, Style};
use bamboo_slip::{Button, RenderError, SettingRow, Switch, Ui, Visual, group, section_title};
use qingjian_platform::LogLevel;

use crate::message::Message;
use crate::system::action::Action;
use crate::view::Page;
use crate::view::parts::actions::{actions, with_notice};
use crate::view::parts::controls::toggle_info;

pub fn advanced(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let general = &page.settings.config.general;
    let mut out = Vec::new();

    let buttons = actions(
        ui,
        page,
        &[
            ("在编辑器中打开", Action::OpenConfig),
            ("打开数据目录", Action::OpenDataDir),
        ],
    )?;
    let row = SettingRow::new("配置文件")
        .control(buttons)
        .build(ui, theme)?;
    let import = Button::new("导入…").build(ui, theme, "import-config", Message::ImportConfig)?;
    let export = Button::new("导出…").build(ui, theme, "export-config", Message::ExportConfig)?;
    let transfer = ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            gap: page.gap(8.0),
            ..Style::default()
        },
        Visual::Group,
        &[import, export],
    )?;
    let transfer = with_notice(
        ui,
        page,
        SettingRow::new("导入、导出设置")
            .info("换电脑或备份时把设置存成一个文件；不含自定义短语（在「自定义短语」页另行导入导出）、云服务密钥与状态条位置")
            .control(transfer),
        "config-transfer",
    )?;
    out.push(section_title(ui, theme, "配置")?);
    out.push(group(ui, theme, &[row, transfer])?);

    let verbose = general.log_level == LogLevel::Debug;
    let next = if verbose {
        LogLevel::Info
    } else {
        LogLevel::Debug
    };
    let switch = Switch::new(verbose).build(
        ui,
        theme,
        "verbose-log",
        Message::set("general", "log_level", next.key()),
    )?;
    let buttons = actions(
        ui,
        page,
        &[
            ("打开日志目录", Action::OpenLogDir),
            ("打包到桌面", Action::ExportLogs),
        ],
    )?;
    let rows = vec![
        SettingRow::new("详细日志")
            .info("会记下敲的内容，排查完关掉")
            .control(switch)
            .build(ui, theme)?,
        with_notice(
            ui,
            page,
            SettingRow::new("日志文件").control(buttons),
            "logs",
        )?,
    ];
    out.push(section_title(ui, theme, "日志")?);
    out.push(group(ui, theme, &rows)?);

    let armed = page.settings.armed == Some(Action::ClearInputLog);
    let mut clear = Button::new(if armed {
        "再点一次清空"
    } else {
        "清空"
    });
    if armed {
        clear = clear.primary();
    }
    let clear = clear.build(
        ui,
        theme,
        "clear-input-log",
        Message::Run(Action::ClearInputLog),
    )?;
    let rows = vec![
        toggle_info(
            ui,
            page,
            "记录输入日志",
            Some("只存在这台电脑上"),
            ("general", "input_log"),
            general.input_log,
            false,
        )?,
        with_notice(
            ui,
            page,
            SettingRow::new("清空输入日志").control(clear),
            "input-log",
        )?,
    ];
    out.push(section_title(ui, theme, "输入日志")?);
    out.push(group(ui, theme, &rows)?);
    Ok(out)
}
