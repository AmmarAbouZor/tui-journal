use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralStyles {
    // input text
    #[serde(default = "input_block_active")]
    pub input_block_active: Style,
    #[serde(default = "input_block_invalid")]
    pub input_block_invalid: Style,
    #[serde(default = "input_corsur_active")]
    pub input_corsur_active: Style,
    #[serde(default = "input_corsur_invalid")]
    pub input_corsur_invalid: Style,

    // General list items
    #[serde(default = "list_item_selected")]
    pub list_item_selected: Style,
    #[serde(default = "list_highlight_active")]
    pub list_highlight_active: Style,
    #[serde(default = "list_highlight_inactive")]
    pub list_highlight_inactive: Style,
}

impl Default for GeneralStyles {
    fn default() -> Self {
        Self {
            input_block_active: input_block_active(),
            input_block_invalid: input_block_invalid(),
            input_corsur_active: input_corsur_active(),
            input_corsur_invalid: input_corsur_invalid(),
            list_item_selected: list_item_selected(),
            list_highlight_active: list_highlight_active(),
            list_highlight_inactive: list_highlight_inactive(),
        }
    }
}

#[inline]
fn input_block_invalid() -> Style {
    Style {
        fg: Some(INVALID_CONTROL_COLOR),
        ..Default::default()
    }
}

#[inline]
fn input_block_active() -> Style {
    Style {
        fg: Some(ACTIVE_INPUT_BORDER_COLOR),
        ..Default::default()
    }
}

#[inline]
fn input_corsur_active() -> Style {
    Style {
        bg: Some(ACTIVE_INPUT_BORDER_COLOR),
        fg: Some(Color::Black),
        ..Default::default()
    }
}

#[inline]
fn input_corsur_invalid() -> Style {
    Style {
        bg: Some(INVALID_CONTROL_COLOR),
        fg: Some(Color::Black),
        ..Default::default()
    }
}

#[inline]
fn list_item_selected() -> Style {
    Style {
        fg: Some(Color::LightYellow),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}
fn list_highlight_active() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(Color::LightGreen),
        ..Default::default()
    }
}

fn list_highlight_inactive() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(Color::LightBlue),
        ..Default::default()
    }
}
