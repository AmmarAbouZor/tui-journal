use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

use crate::app::ui::{ACTIVE_CONTROL_COLOR, INACTIVE_CONTROL_COLOR};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalsListStyles {
    pub block_active: Style,
    pub block_inactive: Style,
    pub block_multi_select: Style,
    pub highlight_active: Style,
    pub highlight_inactive: Style,
    pub title_active: Style,
    pub title_inactive: Style,
    /// Styles when item is marked as selected in select mode
    pub title_selected: Style,
    pub date_priority: Style,
    pub tags_default: Style,
}

impl Default for JournalsListStyles {
    fn default() -> Self {
        let block_active = Style {
            fg: Some(ACTIVE_CONTROL_COLOR),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };
        let block_inactive = Style {
            fg: Some(INACTIVE_CONTROL_COLOR),
            ..Default::default()
        };
        let block_multi_select = Style {
            fg: Some(SELECTED_FOREGROUND_COLOR),
            modifiers: Modifier::BOLD | Modifier::ITALIC,
            ..Default::default()
        };

        let highlight_active = Style {
            fg: Some(Color::Black),
            bg: Some(Color::LightGreen),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };

        let highlight_inactive = Style {
            fg: Some(Color::Black),
            bg: Some(Color::LightBlue),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };

        let title_active = Style {
            fg: Some(ACTIVE_CONTROL_COLOR),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };
        let title_inactive = Style {
            fg: Some(INACTIVE_CONTROL_COLOR),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };
        let title_selected = Style {
            fg: Some(SELECTED_FOREGROUND_COLOR),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };

        let date_priority = Style {
            fg: Some(Color::LightBlue),
            ..Default::default()
        };

        let tags_default = Style {
            fg: Some(Color::LightCyan),
            modifiers: Modifier::DIM,
            ..Default::default()
        };

        Self {
            block_active,
            block_inactive,
            block_multi_select,
            highlight_active,
            highlight_inactive,
            title_active,
            title_inactive,
            title_selected,
            date_priority,
            tags_default,
        }
    }
}
