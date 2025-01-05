mod editor_styles;
mod general_styles;
mod journals_list_styles;
mod style;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub use editor_styles::EditorStyles;
pub use general_styles::GeneralStyles;
pub use journals_list_styles::JournalsListStyles;
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
}
