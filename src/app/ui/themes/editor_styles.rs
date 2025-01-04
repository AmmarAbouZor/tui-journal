use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorStyles {
    pub block_insert: Style,
    pub block_visual: Style,
    pub block_normal_active: Style,
    pub block_normal_inactive: Style,
    pub cursor_normal: Style,
    pub cursor_insert: Style,
    pub cursor_visual: Style,
    pub selection_style: Style,
}

impl Default for EditorStyles {
    fn default() -> Self {
        let block_insert = Style {
            fg: Some(EDITOR_MODE_COLOR),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };

        let block_visual = Style {
            fg: Some(VISUAL_MODE_COLOR),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };

        let block_normal_active = Style {
            fg: Some(ACTIVE_CONTROL_COLOR),
            modifiers: Modifier::BOLD,
            ..Default::default()
        };

        let block_normal_inactive = Style {
            fg: Some(INACTIVE_CONTROL_COLOR),
            ..Default::default()
        };

        let cursor_normal = Style {
            fg: Some(Color::Black),
            bg: Some(Color::White),
            ..Default::default()
        };

        let cursor_insert = Style {
            fg: Some(Color::Black),
            bg: Some(EDITOR_MODE_COLOR),
            ..Default::default()
        };

        let cursor_visual = Style {
            fg: Some(Color::Black),
            bg: Some(VISUAL_MODE_COLOR),
            ..Default::default()
        };

        let selection_style = Style {
            fg: Some(Color::Black),
            bg: Some(Color::White),
            ..Default::default()
        };

        Self {
            block_insert,
            block_visual,
            block_normal_active,
            block_normal_inactive,
            cursor_normal,
            cursor_insert,
            cursor_visual,
            selection_style,
        }
    }
}
