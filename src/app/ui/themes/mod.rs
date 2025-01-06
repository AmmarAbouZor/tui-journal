mod editor_styles;
mod general_styles;
mod journals_list_styles;
mod msgbox;
mod style;

use std::{fs, path::PathBuf};

use anyhow::Context;
use directories::BaseDirs;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub use editor_styles::EditorStyles;
pub use general_styles::GeneralStyles;
pub use journals_list_styles::JournalsListStyles;
pub use msgbox::MsgBoxColors;
pub use style::Style;

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
    pub fn file_path() -> anyhow::Result<PathBuf> {
        BaseDirs::new()
            .map(|base_dirs| {
                base_dirs
                    .config_dir()
                    .join("tui-journal")
                    .join("themes.toml")
            })
            .context("Themes file path couldn't be retrieved")
    }

    /// Serialize default themes to `toml` format.
    pub fn serialize_default() -> anyhow::Result<String> {
        let def_style = Self::default();
        toml::to_string_pretty(&def_style)
            .context("Error while serializing default styles to toml format")
    }

    pub fn load() -> anyhow::Result<Self> {
        let file_path = Self::file_path()?;
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
    use ratatui::style::Modifier;

    use super::*;

    #[test]
    fn full_general_only() {
        let text = r##"
[general.input_block_active]
fg = "Yellow"
modifiers = "DIM"

[general.input_block_invalid]
fg = "Red"
modifiers = ""

[general.input_corsur_active]
fg = "Black"
bg = "LightYellow"
modifiers = ""

[general.input_corsur_invalid]
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
            style.general.input_corsur_invalid,
            def_general.input_corsur_invalid
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
            style.general.input_corsur_invalid,
            def_general.input_corsur_invalid
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
