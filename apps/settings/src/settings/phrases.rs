//! 自定义短语的编辑卡片（打开、关闭、存回配置）与导入导出。

use bamboo_slip::TextEdit;
use qingjian_core::CustomPhrase;
use qingjian_core::custom_phrase::{DEFAULT_WEIGHT, merge_imported, parse_weight};
use qingjian_platform::{Config, export_phrases, import_phrases};

use super::Settings;
use crate::message::FieldId;
use crate::phrase_editor::PhraseEditor;

impl Settings {
    pub(super) fn open_phrase(&mut self, index: Option<usize>) {
        let (editor, code, weight, text) =
            match index.and_then(|i| Some((i, self.config.custom_phrases.get(i)?))) {
                Some((i, phrase)) => (
                    PhraseEditor::edit(i, phrase),
                    phrase.code.clone(),
                    phrase.weight,
                    phrase.text.clone(),
                ),
                None => (
                    PhraseEditor::new(),
                    String::new(),
                    DEFAULT_WEIGHT,
                    String::new(),
                ),
            };
        self.fields.insert(FieldId::PhraseCode, TextEdit::new(code));
        self.fields
            .insert(FieldId::PhraseWeight, TextEdit::new(weight.to_string()));
        self.fields
            .insert(FieldId::PhraseText, TextEdit::multiline(text));
        self.phrase = Some(editor);
    }

    pub(super) fn close_phrase(&mut self) {
        self.phrase = None;
        self.fields.remove(&FieldId::PhraseCode);
        self.fields.remove(&FieldId::PhraseWeight);
        self.fields.remove(&FieldId::PhraseText);
    }

    /// 存卡片（`delete` 时删掉这条）：整张表校验写回，失败把原因留在卡片里。
    pub(super) fn save_phrase(&mut self, delete: bool) {
        let Some(editor) = self.phrase.clone() else {
            return;
        };
        let phrase = if delete {
            Ok(None)
        } else {
            self.phrase_draft(&editor).map(Some)
        };
        let saved = phrase
            .and_then(|phrase| editor.apply(&self.config.custom_phrases, phrase))
            .and_then(|list| self.write_phrases(&list));
        match saved {
            Ok(()) => self.close_phrase(),
            Err(error) => {
                if let Some(editor) = self.phrase.as_mut() {
                    editor.error = Some(error);
                    editor.confirm_delete = false;
                }
            }
        }
    }

    /// 卡片里填的这一条；权重不是范围内的整数时报错。
    fn phrase_draft(&self, editor: &PhraseEditor) -> Result<CustomPhrase, String> {
        let field = |id| {
            self.fields
                .get(&id)
                .map(|edit: &TextEdit| edit.text().to_owned())
                .unwrap_or_default()
        };
        Ok(CustomPhrase {
            code: field(FieldId::PhraseCode).trim().to_owned(),
            text: field(FieldId::PhraseText),
            weight: parse_weight(&field(FieldId::PhraseWeight))?,
            enabled: editor.enabled,
        })
    }

    /// 选一个导出的文件（或另一台电脑的整份配置文件），短语并进现有的；同码同文本的不重复加。
    pub(super) fn import_phrases(&mut self) {
        let Some(source) = rfd::FileDialog::new()
            .set_title("导入自定义短语")
            .add_filter("自定义短语或配置文件", &["toml"])
            .add_filter("所有文件", &["*"])
            .pick_file()
        else {
            return;
        };
        let result = std::fs::read_to_string(&source)
            .map_err(|error| format!("读不了这个文件：{error}"))
            .and_then(|text| import_phrases(&text))
            .and_then(|(imported, skipped)| {
                let total = imported.len();
                let (list, added) = merge_imported(&self.config.custom_phrases, imported);
                if added > 0 {
                    self.write_phrases(&list)?;
                }
                let mut summary = format!("新加 {added} 条");
                if total > added {
                    summary.push_str(&format!("，{} 条已经有了", total - added));
                }
                if skipped > 0 {
                    summary.push_str(&format!("，{skipped} 条写错跳过"));
                }
                Ok(summary)
            });
        self.notice = Some(("phrases-transfer", result.unwrap_or_else(|error| error)));
    }

    /// 全部短语（含关着的）存成一个文件，换电脑或备份时在另一边导入。
    pub(super) fn export_phrases(&mut self) {
        let phrases = &self.config.custom_phrases;
        if phrases.is_empty() {
            self.notice = Some(("phrases-transfer", "还没有短语可导出".to_owned()));
            return;
        }
        let name = format!("青简自定义短语-{}.toml", jiff::Zoned::now().date());
        let Some(target) = rfd::FileDialog::new()
            .set_title("导出自定义短语")
            .set_file_name(&name)
            .add_filter("自定义短语", &["toml"])
            .save_file()
        else {
            return;
        };
        let notice = match std::fs::write(&target, export_phrases(phrases)) {
            Ok(()) => format!("已导出 {} 条到 {}", phrases.len(), target.display()),
            Err(error) => format!("写不进去：{error}"),
        };
        self.notice = Some(("phrases-transfer", notice));
    }

    pub(super) fn write_phrases(&mut self, list: &[CustomPhrase]) -> Result<(), String> {
        Config::set_custom_phrases(&self.path, list)?;
        self.reload();
        Ok(())
    }
}
