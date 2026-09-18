//! 「候选窗口」页：外观、排布、渲染引擎、字体（可搜索的列表）、拼音显示位置。

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::NSPopUpButton;
use qingjian_platform::{Appearance, CandidateRenderer, Config, LayoutMode, PreeditMode};
use qingjian_render::ThemeLibrary;

use crate::candidates::available_families;
use crate::preferences::controls::{note, row_popup, select, set_items};
use crate::preferences::font_picker::FontPicker;
use crate::preferences::layout::Layout;
use crate::preferences::setting::Setting;
use crate::preferences::target::PreferencesTarget;

pub struct CandidatesPage {
    /// 外观：跟随系统 / 浅色 / 深色。
    appearance: Retained<NSPopUpButton>,

    /// 主题：内置主题按显示名列出。
    theme: Retained<NSPopUpButton>,

    /// 竖排 / 横排。
    layout_mode: Retained<NSPopUpButton>,

    /// 青简渲染器 / 系统绘制。
    renderer: Retained<NSPopUpButton>,

    /// 候选窗字体：搜索框 + 列表。
    font: FontPicker,

    /// 拼音显示位置。
    preedit: Retained<NSPopUpButton>,
}

impl CandidatesPage {
    pub fn build(layout: &mut Layout, mtm: MainThreadMarker, target: &PreferencesTarget) -> Self {
        let appearance_titles: Vec<String> = Appearance::ALL
            .iter()
            .map(|a| a.label().to_owned())
            .collect();
        let appearance = row_popup(
            layout,
            mtm,
            "外观",
            &appearance_titles,
            Setting::Appearance,
            target,
        );
        let theme = row_popup(
            layout,
            mtm,
            "主题",
            &theme_titles(&themes()),
            Setting::Theme,
            target,
        );
        note(
            layout,
            mtm,
            "主题只对青简渲染器生效；每个主题都有浅色与深色两套，按上面的外观切换。",
        );
        let layout_titles: Vec<String> = LayoutMode::ALL
            .iter()
            .map(|l| l.label().to_owned())
            .collect();
        let layout_mode = row_popup(layout, mtm, "排布", &layout_titles, Setting::Layout, target);
        note(layout, mtm, "横排时只给高亮的候选显示译词。");
        let renderer_titles: Vec<String> = CandidateRenderer::ALL
            .iter()
            .map(|r| r.label().to_owned())
            .collect();
        let renderer = row_popup(
            layout,
            mtm,
            "渲染引擎",
            &renderer_titles,
            Setting::Renderer,
            target,
        );
        note(layout, mtm, "青简渲染器让候选窗口在各平台一致。");
        let font = FontPicker::build(layout, mtm, "字体", available_families(mtm));
        note(
            layout,
            mtm,
            "只对青简渲染器生效；没装的字体自动回到系统字体。",
        );
        let preedit_titles: Vec<String> = PreeditMode::ALL
            .iter()
            .map(|p| p.label().to_owned())
            .collect();
        let preedit = row_popup(
            layout,
            mtm,
            "拼音显示",
            &preedit_titles,
            Setting::Preedit,
            target,
        );
        note(
            layout,
            mtm,
            "「只在候选窗口」时正在敲的拼音不显示在应用里，终端或行内拼音显示不正常的应用可以选它。",
        );
        Self {
            appearance,
            theme,
            layout_mode,
            renderer,
            font,
            preedit,
        }
    }

    pub fn sync(&self, config: &Config) {
        let general = &config.general;
        select(
            &self.appearance,
            Appearance::ALL
                .iter()
                .position(|a| *a == general.appearance()),
        );
        // 主题目录里可能新放了主题，每次同步都重列
        let themes = themes();
        set_items(&self.theme, &theme_titles(&themes));
        select(
            &self.theme,
            themes
                .themes()
                .iter()
                .position(|t| t.id() == general.theme_id()),
        );
        select(
            &self.layout_mode,
            LayoutMode::ALL.iter().position(|l| *l == general.layout),
        );
        select(
            &self.renderer,
            CandidateRenderer::ALL
                .iter()
                .position(|r| *r == general.renderer),
        );
        self.font.sync(&general.font);
        select(
            &self.preedit,
            PreeditMode::ALL.iter().position(|p| *p == general.preedit),
        );
    }
}

/// 主题库：内置主题加用户主题目录里的。
fn themes() -> ThemeLibrary {
    ThemeLibrary::load(crate::app::paths::themes_dir().as_deref())
}

/// 主题下拉的选项：显示名。
fn theme_titles(themes: &ThemeLibrary) -> Vec<String> {
    themes
        .themes()
        .iter()
        .map(|theme| theme.name().to_owned())
        .collect()
}
