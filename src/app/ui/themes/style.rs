use ratatui::style::{Color, Modifier, Style as RataStyle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Default)]
/// Represents the style elements as they defined in [`ratatui`] lib but with more simple
/// definitions for serialization so it's less confusing for users to define in custom themes.
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    #[serde(default)]
    pub modifiers: Modifier,
    pub underline_color: Option<Color>,
}

impl From<Style> for RataStyle {
    fn from(style: Style) -> Self {
        let mut rata_style = RataStyle {
            bg: style.bg,
            fg: style.fg,
            underline_color: style.underline_color,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        };

        // Modifiers needed to be added via this method to be involved in both add and sub modifier
        rata_style = rata_style.add_modifier(style.modifiers);
        rata_style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_style_json() {
        let style_json = r##"{
  "fg": "LightCyan",
  "bg": "#4F11BA",
  "modifiers": "DIM | UNDERLINED",
  "underline_color": "#0A1432"
}"##;

        let style = Style {
            fg: Some(Color::LightCyan),
            bg: Some(Color::from_u32(0x004F11BA)),
            underline_color: Some(Color::Rgb(10, 20, 50)),
            modifiers: Modifier::UNDERLINED | Modifier::DIM,
        };

        let serialized = serde_json::from_str(style_json).unwrap();
        assert_eq!(style, serialized);
    }

    #[test]
    fn serialize_style_toml() {
        let style_toml = r##"
fg = "LightCyan"
bg = "#4F11BA"
modifiers = ""
underline_color = "#0A1432"
"##;

        let style = Style {
            fg: Some(Color::LightCyan),
            bg: Some(Color::from_u32(0x004F11BA)),
            underline_color: Some(Color::Rgb(10, 20, 50)),
            modifiers: Modifier::empty(),
        };

        let serialized = toml::from_str(style_toml).unwrap();
        assert_eq!(style, serialized);
    }

    #[test]
    fn serialized_style_missing() {
        let style_toml = r##"
fg = "LightCyan"
bg = "#4F11BA"
"##;

        let style = Style {
            fg: Some(Color::LightCyan),
            bg: Some(Color::from_u32(0x004F11BA)),
            underline_color: None,
            modifiers: Modifier::empty(),
        };

        let serialized = toml::from_str(style_toml).unwrap();

        assert_eq!(style, serialized);
    }
}
