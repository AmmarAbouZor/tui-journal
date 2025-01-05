use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralStyles {
    // input text
    pub input_block_active: Style,
    pub input_block_invalid: Style,
    pub input_corsur_active: Style,
    pub input_corsur_invalid: Style,

    // General list items
    pub list_item_selected: Style,
    pub list_highlight_active: Style,
    pub list_highlight_inactive: Style,
}

impl Default for GeneralStyles {
    fn default() -> Self {
        let input_block_invalid = Style {
            fg: Some(INVALID_CONTROL_COLOR),
            ..Default::default()
        };
        let input_block_active = Style {
            fg: Some(ACTIVE_INPUT_BORDER_COLOR),
            ..Default::default()
        };
        let input_corsur_active = Style {
            bg: Some(ACTIVE_INPUT_BORDER_COLOR),
            fg: Some(Color::Black),
            ..Default::default()
        };
        let input_corsur_invalid = Style {
            bg: Some(INVALID_CONTROL_COLOR),
            fg: Some(Color::Black),
            ..Default::default()
        };

        let list_item_selected = Style {
            fg: Some(Color::LightYellow),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };
        let list_highlight_active = Style {
            fg: Some(Color::Black),
            bg: Some(Color::LightGreen),
            ..Default::default()
        };

        let list_highlight_inactive = Style {
            fg: Some(Color::Black),
            bg: Some(Color::LightBlue),
            ..Default::default()
        };

        Self {
            input_block_active,
            input_block_invalid,
            input_corsur_active,
            input_corsur_invalid,
            list_item_selected,
            list_highlight_active,
            list_highlight_inactive,
        }
    }
}
