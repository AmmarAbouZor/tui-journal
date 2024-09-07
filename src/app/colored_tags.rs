use std::collections::HashMap;

use ratatui::style::Color;

/// Hard coded colors for the tags.
/// Note: the order to pick the colors is from bottom to top because we are popping the colors from
/// the end of the stack.
const TAG_COLORS: &[TagColor] = &[
    TagColor::new(Color::Black, Color::LightMagenta),
    TagColor::new(Color::Red, Color::Cyan),
    TagColor::new(Color::Yellow, Color::Blue),
    TagColor::new(Color::Reset, Color::Red),
    TagColor::new(Color::Black, Color::LightYellow),
    TagColor::new(Color::Reset, Color::DarkGray),
    TagColor::new(Color::Black, Color::LightGreen),
    TagColor::new(Color::Black, Color::LightRed),
    TagColor::new(Color::Black, Color::LightCyan),
];

#[derive(Debug, Clone)]
/// Manages assigning colors to the tags, keeping track on the assigned colors and providing
/// functions to updating them.
pub struct ColoredTagsManager {
    tag_colors_map: HashMap<String, TagColor>,
    available_colors: Vec<TagColor>,
}

impl ColoredTagsManager {
    pub fn new() -> Self {
        let available_colors = TAG_COLORS.to_vec();

        Self {
            tag_colors_map: HashMap::new(),
            available_colors,
        }
    }

    /// Updates the tag_color map with the provided tags, removing the not existing tags and
    /// assigning colors to the newly added ones.
    pub fn update_tags(&mut self, current_tags: Vec<String>) {
        // First: Clear the non-existing anymore tags.
        let tags_to_remove: Vec<_> = self
            .tag_colors_map
            .keys()
            .filter(|t| !current_tags.contains(t))
            .cloned()
            .collect();

        for tag in tags_to_remove {
            let color = self.tag_colors_map.remove(&tag).unwrap();
            self.available_colors.push(color)
        }

        // Second: Add the new tags to the map
        for tag in current_tags {
            match self.tag_colors_map.entry(tag) {
                std::collections::hash_map::Entry::Occupied(_) => {}
                std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                    let color = self.available_colors.pop().unwrap_or_default();
                    vacant_entry.insert(color);
                }
            }
        }
    }

    /// Gets the matching color for the giving tag if tag exists.
    pub fn get_tag_color(&self, tag: &str) -> Option<TagColor> {
        self.tag_colors_map.get(tag).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TagColor {
    pub foreground: Color,
    pub background: Color,
}

impl TagColor {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self {
            foreground,
            background,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_colored_tags() {
        const TAG_ONE: &str = "Tag 1";
        const TAG_TWO: &str = "Tag 2";
        const ADDED_TAG: &str = "Added Tag";

        let mut tags = vec![
            String::from(TAG_ONE),
            String::from(TAG_TWO),
            String::from("Tag 3"),
            String::from("Tag 4"),
        ];

        let mut manager = ColoredTagsManager::new();
        manager.update_tags(tags.clone());

        // Ensure all tags have colors.
        for tag in tags.iter() {
            assert!(manager.get_tag_color(tag).is_some());
        }

        // Ensure non existing tags are none
        assert!(manager.get_tag_color("Non Existing Tag").is_none());

        // Keep track on colors before updating.
        let tag_one_color = manager.get_tag_color(TAG_ONE).unwrap();
        let tag_two_color = manager.get_tag_color(TAG_TWO).unwrap();

        // Remove Tag one with changing the order of the tags.
        assert_eq!(tags.swap_remove(0), TAG_ONE);

        tags.push(ADDED_TAG.into());

        manager.update_tags(tags.clone());

        // Ensure all current tags have colors.
        for tag in tags.iter() {
            assert!(manager.get_tag_color(tag).is_some());
        }

        // Tag one should have no color after remove.
        assert!(manager.get_tag_color(TAG_ONE).is_none());

        // Tag two color must remain the same after update.
        assert_eq!(manager.get_tag_color(TAG_TWO).unwrap(), tag_two_color);

        // Added tag should take the color of tag one because we removed it then added the new tag.
        assert_eq!(manager.get_tag_color(ADDED_TAG).unwrap(), tag_one_color);
    }
}
