use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalsListStyles {
    #[serde(default = "block_active")]
    pub block_active: Style,
    #[serde(default = "block_inactive")]
    pub block_inactive: Style,
    #[serde(default = "block_multi_select")]
    pub block_multi_select: Style,
    #[serde(default = "highlight_active")]
    pub highlight_active: Style,
    #[serde(default = "highlight_inactive")]
    pub highlight_inactive: Style,
    #[serde(default = "title_active")]
    pub title_active: Style,
    #[serde(default = "title_inactive")]
    pub title_inactive: Style,
    /// Styles when item is marked as selected in select mode
    #[serde(default = "title_selected")]
    pub title_selected: Style,
    #[serde(default = "date_priority")]
    pub date_priority: Style,
    #[serde(default = "tags_default")]
    pub tags_default: Style,
}

impl Default for JournalsListStyles {
    fn default() -> Self {
        Self {
            block_active: block_active(),
            block_inactive: block_inactive(),
            block_multi_select: block_multi_select(),
            highlight_active: highlight_active(),
            highlight_inactive: highlight_inactive(),
            title_active: title_active(),
            title_inactive: title_inactive(),
            title_selected: title_selected(),
            date_priority: date_priority(),
            tags_default: tags_default(),
        }
    }
}

#[inline]
fn block_active() -> Style {
    Style {
        fg: Some(ACTIVE_CONTROL_COLOR),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn block_inactive() -> Style {
    Style {
        fg: Some(INACTIVE_CONTROL_COLOR),
        ..Default::default()
    }
}

#[inline]
fn block_multi_select() -> Style {
    Style {
        fg: Some(SELECTED_FOREGROUND_COLOR),
        modifiers: Modifier::BOLD | Modifier::ITALIC,
        ..Default::default()
    }
}

#[inline]
fn highlight_active() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(Color::LightGreen),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn highlight_inactive() -> Style {
    Style {
        fg: Some(Color::Black),
        bg: Some(Color::LightBlue),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn title_active() -> Style {
    Style {
        fg: Some(ACTIVE_CONTROL_COLOR),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn title_inactive() -> Style {
    Style {
        fg: Some(INACTIVE_CONTROL_COLOR),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn title_selected() -> Style {
    Style {
        fg: Some(SELECTED_FOREGROUND_COLOR),
        modifiers: Modifier::BOLD,
        ..Default::default()
    }
}

#[inline]
fn date_priority() -> Style {
    Style {
        fg: Some(Color::LightBlue),
        ..Default::default()
    }
}

#[inline]
fn tags_default() -> Style {
    Style {
        fg: Some(Color::LightCyan),
        modifiers: Modifier::DIM,
        ..Default::default()
    }
}
