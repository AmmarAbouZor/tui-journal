use std::collections::BTreeMap;

use ratatui::widgets::TableState;

use crate::app::{
    keymap::{get_multi_select_keymaps, Input},
    ui::UICommand,
};

use super::keybindings_table::KeybindingsTable;

#[derive(Debug)]
pub struct MultiSelectBindings {
    state: TableState,
    bingings_map: BTreeMap<UICommand, Vec<Input>>,
}

impl MultiSelectBindings {
    pub fn new() -> Self {
        let state = TableState::default();
        let mut bingings_map: BTreeMap<UICommand, Vec<Input>> = BTreeMap::new();

        get_multi_select_keymaps().into_iter().for_each(|keymap| {
            bingings_map
                .entry(keymap.command)
                .and_modify(|keys| keys.push(keymap.key))
                .or_insert(vec![keymap.key]);
        });
        Self {
            state,
            bingings_map,
        }
    }
}

impl KeybindingsTable for MultiSelectBindings {
    fn get_state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    fn get_bindings_map(&self) -> &BTreeMap<UICommand, Vec<Input>> {
        &self.bingings_map
    }

    fn get_title(&self) -> &str {
        "Multi-Select Mode Keybindings"
    }
}
