mod editor_styles;
mod general_styles;
mod journals_list_styles;
mod msgbox;
mod serialization;
mod style;

use std::path::PathBuf;

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
    pub general: GeneralStyles,
    pub journals_list: JournalsListStyles,
    pub editor: EditorStyles,
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
}
