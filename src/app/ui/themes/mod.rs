mod editor_styles;
mod general_styles;
mod journals_list_styles;
mod msgbox;
mod style;

use std::{fs, path::PathBuf};

use anyhow::Context;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub use editor_styles::EditorStyles;
pub use general_styles::GeneralStyles;
pub use journals_list_styles::JournalsListStyles;
pub use msgbox::MsgBoxColors;
pub use style::Style;

use crate::settings::settings_default_dir_path;

const ACTIVE_CONTROL_COLOR: Color = Color::Reset;
const INACTIVE_CONTROL_COLOR: Color = Color::Rgb(170, 170, 200);
const EDITOR_MODE_COLOR: Color = Color::LightGreen;
const VISUAL_MODE_COLOR: Color = Color::Blue;
const SELECTED_FOREGROUND_COLOR: Color = Color::Yellow;
const INVALID_CONTROL_COLOR: Color = Color::LightRed;
const ACTIVE_INPUT_BORDER_COLOR: Color = Color::LightYellow;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Styles {
    #[serde(default)]
    pub general: GeneralStyles,
    #[serde(default)]
    pub journals_list: JournalsListStyles,
    #[serde(default)]
    pub editor: EditorStyles,
    #[serde(default)]
    pub msgbox: MsgBoxColors,
}

impl Styles {
    pub fn file_path(custom_config_dir: Option<&PathBuf>) -> anyhow::Result<PathBuf> {
        let config_dir = match custom_config_dir {
            Some(dir) if dir.is_dir() => dir,
            Some(_path) => {
                // It's possible for users to provide path for configuration file instead of
                // directory because this was the previous API.
                // In this situation it's enough to warn them and ignore the path in themes.
                eprintln!(
                    "INFO: Custom config directory is ignored in themese because it's not a directory."
                );
                log::warn!(
                    "Custom config directory is ignored in themese because it's not a directory"
                );
                &settings_default_dir_path()?
            }
            None => &settings_default_dir_path()?,
        };
        let path = config_dir.join("themes.toml");

        Ok(path)
    }

    /// Serialize default themes to `toml` format.
    pub fn serialize_default() -> anyhow::Result<String> {
        let def_style = Self::default();
        toml::to_string_pretty(&def_style)
            .context("Error while serializing default styles to toml format")
    }

    pub fn load(custom_config_dir: Option<&PathBuf>) -> anyhow::Result<Self> {
        let file_path = Self::file_path(custom_config_dir)?;
        if !file_path.exists() {
            return Ok(Self::default());
        }

        let file_content = fs::read_to_string(&file_path).with_context(|| {
            format!(
                "Loading themes file content failed. Path: {}",
                file_path.display()
            )
        })?;

        Self::deserialize(&file_content).with_context(|| {
            format!(
                "Error while desrializing toml text to styles. File path: {}",
                file_path.display()
            )
        })
    }

    /// Deserialize [`Styles`] from the given text, filling the missing items from default
    /// implementation.
    fn deserialize(input: &str) -> anyhow::Result<Self> {
        toml::from_str(input).map_err(|err| err.into())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ratatui::style::Modifier;

    use super::*;

    #[test]
    fn file_path_uses_directory() {
        let dir = tempfile::Builder::new()
            .prefix("themes-dir")
            .tempdir()
            .unwrap();

        let path = Styles::file_path(Some(&dir.path().to_path_buf())).unwrap();

        assert_eq!(path, dir.path().join("themes.toml"));
    }

    #[test]
    fn file_path_ignores_file_input() {
        let dir = tempfile::Builder::new()
            .prefix("themes-file")
            .tempdir()
            .unwrap();
        let config_file = dir.path().join("config.toml");
        fs::write(&config_file, "").unwrap();

        // Config file paths stay backward compatible, but themes still resolve from the default dir.
        let path = Styles::file_path(Some(&config_file)).unwrap();

        assert!(path.ends_with("themes.toml"));
        assert!(!path.starts_with(&config_file));
    }

    #[test]
    fn load_missing_returns_default() {
        let dir = tempfile::Builder::new()
            .prefix("themes-load")
            .tempdir()
            .unwrap();

        let styles = Styles::load(Some(&dir.path().to_path_buf())).unwrap();

        assert_eq!(styles.general, GeneralStyles::default());
        assert_eq!(styles.journals_list, JournalsListStyles::default());
        assert_eq!(styles.editor, EditorStyles::default());
        assert_eq!(styles.msgbox, MsgBoxColors::default());
    }

    #[test]
    fn serialize_default_round_trips() {
        let text = Styles::serialize_default().unwrap();
        let styles = Styles::deserialize(&text).unwrap();

        assert_eq!(styles.general, GeneralStyles::default());
        assert_eq!(styles.journals_list, JournalsListStyles::default());
        assert_eq!(styles.editor, EditorStyles::default());
        assert_eq!(styles.msgbox, MsgBoxColors::default());
    }

    #[test]
    fn full_general_only() {
        let text = r##"
[general.input_block_active]
fg = "Yellow"
modifiers = "DIM"

[general.input_block_invalid]
fg = "Red"
modifiers = ""

[general.input_cursor_active]
fg = "Black"
bg = "LightYellow"
modifiers = ""

[general.input_cursor_invalid]
fg = "Black"
bg = "LightRed"
modifiers = ""

[general.list_item_selected]
fg = "LightYellow"
modifiers = "BOLD"

[general.list_highlight_active]
fg = "Black"
bg = "LightGreen"
modifiers = ""

[general.list_highlight_inactive]
fg = "Black"
bg = "LightBlue"
modifiers = ""
        "##;

        let style = Styles::deserialize(text).unwrap();
        assert_eq!(style.general.input_block_active.fg, Some(Color::Yellow));
        assert_eq!(style.general.input_block_active.modifiers, Modifier::DIM);
        assert_eq!(style.general.input_block_invalid.fg, Some(Color::Red));

        assert_eq!(style.journals_list, JournalsListStyles::default());
        assert_eq!(style.editor, EditorStyles::default());
        assert_eq!(style.msgbox, MsgBoxColors::default());
    }

    #[test]
    fn compat_general_only() {
        let text = r##"
[general.input_corsur_active]
fg = "Black"
bg = "LightYellow"
modifiers = ""

[general.input_corsur_invalid]
fg = "Black"
bg = "LightRed"
modifiers = ""
        "##;

        let style = Styles::deserialize(text).unwrap();
        assert_eq!(style.general.input_cursor_active.fg, Some(Color::Black));
        assert_eq!(style.general.input_cursor_active.bg, Some(Color::LightYellow));
        assert_eq!(style.general.input_cursor_invalid.fg, Some(Color::Black));
        assert_eq!(style.general.input_cursor_invalid.bg, Some(Color::LightRed));

        assert_eq!(style.journals_list, JournalsListStyles::default());
        assert_eq!(style.editor, EditorStyles::default());
        assert_eq!(style.msgbox, MsgBoxColors::default());
    }

    #[test]
    fn compat_general_both() {
        let text = r##"
[general.input_corsur_active]
fg = "Black"
bg = "LightYellow"
modifiers = ""

[general.input_cursor_active]
fg = "Black"
bg = "LightRed"
modifiers = ""
        "##;

        let err = Styles::deserialize(text).unwrap_err();
        assert_eq!(
            format!("{err}"),
            "TOML parse error at line 2, column 2\n  \
               |\n\
             2 | [general.input_corsur_active]\n  \
               |  ^^^^^^^\n\
             duplicate field `input_cursor_active`\n"
        );
    }

    #[test]
    fn part_general_only() {
        let text = r##"
[general.input_block_active]
fg = "Yellow"
modifiers = "DIM"

[general.input_block_invalid]
fg = "Red"
modifiers = ""
        "##;

        let style = Styles::deserialize(text).unwrap();
        assert_eq!(style.general.input_block_active.fg, Some(Color::Yellow));
        assert_eq!(style.general.input_block_active.modifiers, Modifier::DIM);
        assert_eq!(style.general.input_block_invalid.fg, Some(Color::Red));

        let def_general = GeneralStyles::default();
        assert_eq!(
            style.general.input_cursor_invalid,
            def_general.input_cursor_invalid
        );

        assert_eq!(style.journals_list, JournalsListStyles::default());
        assert_eq!(style.editor, EditorStyles::default());
        assert_eq!(style.msgbox, MsgBoxColors::default());
    }

    #[test]
    fn part_journals_list_only() {
        let text = r##"
[journals_list.block_active]
fg = "Red"
modifiers = "ITALIC"

[journals_list.block_inactive]
fg = "Reset"
        "##;

        let style = Styles::deserialize(text).unwrap();
        assert_eq!(style.journals_list.block_active.fg, Some(Color::Red));
        assert_eq!(style.journals_list.block_active.modifiers, Modifier::ITALIC);
        assert_eq!(style.journals_list.block_inactive.fg, Some(Color::Reset));

        let def_journals = JournalsListStyles::default();
        assert_eq!(
            style.journals_list.highlight_active,
            def_journals.highlight_active
        );
        assert_eq!(
            style.journals_list.highlight_inactive,
            def_journals.highlight_inactive
        );

        assert_eq!(style.general, GeneralStyles::default());
        assert_eq!(style.editor, EditorStyles::default());
        assert_eq!(style.msgbox, MsgBoxColors::default());
    }

    #[test]
    fn part_editor_only() {
        let text = r##"
[editor.block_insert]
fg = "Blue"
modifiers = ""

[editor.block_visual]
fg = "Red"
        "##;

        let style = Styles::deserialize(text).unwrap();
        assert_eq!(style.editor.block_insert.fg, Some(Color::Blue));
        assert_eq!(style.editor.block_insert.modifiers, Modifier::empty());
        assert_eq!(style.editor.block_visual.fg, Some(Color::Red));

        let def_editor = EditorStyles::default();
        assert_eq!(style.editor.cursor_visual, def_editor.cursor_visual);
        assert_eq!(style.editor.cursor_normal, def_editor.cursor_normal);

        assert_eq!(style.journals_list, JournalsListStyles::default());
        assert_eq!(style.general, GeneralStyles::default());
        assert_eq!(style.msgbox, MsgBoxColors::default());
    }

    #[test]
    fn part_msg_only() {
        let text = r##"
[msgbox]
error = "Red"
warning = "Blue"
        "##;

        let style = Styles::deserialize(text).unwrap();
        assert_eq!(style.msgbox.error, Color::Red);
        assert_eq!(style.msgbox.warning, Color::Blue);

        let def_msg = MsgBoxColors::default();
        assert_eq!(style.msgbox.info, def_msg.info);
        assert_eq!(style.msgbox.question, def_msg.question);

        assert_eq!(style.general, GeneralStyles::default());
        assert_eq!(style.journals_list, JournalsListStyles::default());
        assert_eq!(style.editor, EditorStyles::default());
    }

    #[test]
    /// Tests input have a part of every style group
    fn part_from_all() {
        let text = r##"
[general.input_block_active]
fg = "Yellow"
modifiers = "DIM"

[general.input_block_invalid]
fg = "Red"
modifiers = ""

[journals_list.block_active]
fg = "Red"
modifiers = "ITALIC"

[journals_list.block_inactive]
fg = "Reset"

[editor.block_insert]
fg = "Blue"
modifiers = ""

[editor.block_visual]
fg = "Red"

[msgbox]
error = "Red"
warning = "Blue"
        "##;

        // General
        let style = Styles::deserialize(text).unwrap();
        assert_eq!(style.general.input_block_active.fg, Some(Color::Yellow));
        assert_eq!(style.general.input_block_active.modifiers, Modifier::DIM);
        assert_eq!(style.general.input_block_invalid.fg, Some(Color::Red));

        let def_general = GeneralStyles::default();
        assert_eq!(
            style.general.input_cursor_invalid,
            def_general.input_cursor_invalid
        );

        // Journals
        assert_eq!(style.journals_list.block_active.fg, Some(Color::Red));
        assert_eq!(style.journals_list.block_active.modifiers, Modifier::ITALIC);
        assert_eq!(style.journals_list.block_inactive.fg, Some(Color::Reset));

        let def_journals = JournalsListStyles::default();
        assert_eq!(
            style.journals_list.highlight_active,
            def_journals.highlight_active
        );
        assert_eq!(
            style.journals_list.highlight_inactive,
            def_journals.highlight_inactive
        );

        // Editor
        assert_eq!(style.editor.block_insert.fg, Some(Color::Blue));
        assert_eq!(style.editor.block_insert.modifiers, Modifier::empty());
        assert_eq!(style.editor.block_visual.fg, Some(Color::Red));

        let def_editor = EditorStyles::default();
        assert_eq!(style.editor.cursor_visual, def_editor.cursor_visual);
        assert_eq!(style.editor.cursor_normal, def_editor.cursor_normal);

        // Colors
        let def_msg = MsgBoxColors::default();
        assert_eq!(style.msgbox.error, Color::Red);
        assert_eq!(style.msgbox.warning, Color::Blue);

        assert_eq!(style.msgbox.info, def_msg.info);
        assert_eq!(style.msgbox.question, def_msg.question);
    }
}
