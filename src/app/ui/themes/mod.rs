mod editor_styles;
mod style;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub use editor_styles::EditorStyles;
pub use style::Style;

const ACTIVE_CONTROL_COLOR: Color = Color::Reset;
const INACTIVE_CONTROL_COLOR: Color = Color::Rgb(170, 170, 200);
const EDITOR_MODE_COLOR: Color = Color::LightGreen;
const INVALID_CONTROL_COLOR: Color = Color::LightRed;
const VISUAL_MODE_COLOR: Color = Color::Blue;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Styles {
    pub general: GeneralStyles,
    pub journals_list: JournalsListStyles,
    pub editor: EditorStyles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralStyles {}

impl Default for GeneralStyles {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalsListStyles {}

impl Default for JournalsListStyles {
    fn default() -> Self {
        Self {}
    }
}
