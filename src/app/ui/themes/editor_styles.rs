use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorStyles {
    #[serde(default = "block_insert")]
    pub block_insert: Style,
    #[serde(default = "block_visual")]
    pub block_visual: Style,
    #[serde(default = "block_normal_active")]
    pub block_normal_active: Style,
    #[serde(default = "block_normal_inactive")]
    pub block_normal_inactive: Style,
    #[serde(default = "cursor_normal")]
    pub cursor_normal: Style,
    #[serde(default = "cursor_insert")]
    pub cursor_insert: Style,
    #[serde(default = "cursor_visual")]
    pub cursor_visual: Style,
    #[serde(default = "selection_style")]
    pub selection_style: Style,
}

impl Default for EditorStyles {
    fn default() -> Self {
        Self {
            block_insert: block_insert(),
            block_visual: block_visual(),
            block_normal_active: block_normal_active(),
            block_normal_inactive: block_normal_inactive(),
            cursor_normal: cursor_normal(),
            cursor_insert: cursor_insert(),
            cursor_visual: cursor_visual(),
            selection_style: selection_style(),
        }
    }
}

#[inline]
fn block_insert() -> Style {
    Style {
        fg: Some(EDITOR_MODE_COLOR),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn block_visual() -> Style {
    Style {
        fg: Some(VISUAL_MODE_COLOR),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn block_normal_active() -> Style {
    Style {
        fg: Some(ACTIVE_CONTROL_COLOR),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn block_normal_inactive() -> Style {
    Style {
        fg: Some(INACTIVE_CONTROL_COLOR),
        ..Default::default()
    }
}

#[inline]
fn cursor_normal() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(Color::White),
        ..Default::default()
    }
}

#[inline]
fn cursor_insert() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(EDITOR_MODE_COLOR),
        ..Default::default()
    }
}

#[inline]
fn cursor_visual() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(VISUAL_MODE_COLOR),
        ..Default::default()
    }
}

#[inline]
fn selection_style() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(Color::White),
        ..Default::default()
    }
}
