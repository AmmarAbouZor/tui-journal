use std::collections::BTreeMap;

use ratatui::widgets::TableState;

use crate::app::{
    keymap::{
        get_editor_mode_keymaps, get_entries_list_keymaps, get_global_keymaps, Input, Keymap,
    },
    ui::UICommand,
};

use super::keybindings_table::KeybindingsTable;

#[derive(Debug)]
pub struct GlobalBindings {
    state: TableState,
    bindings_map: BTreeMap<UICommand, Vec<Input>>,
}

impl GlobalBindings {
    pub fn new() -> Self {
        let state = TableState::default();

        let mut bindings_map: BTreeMap<UICommand, Vec<Input>> = BTreeMap::new();

        get_all_keymaps().for_each(|keymap| {
            bindings_map
                .entry(keymap.command)
                .and_modify(|keys| keys.push(keymap.key))
                .or_insert(vec![keymap.key]);
        });

        Self {
            state,
            bindings_map,
        }
    }
}

fn get_all_keymaps() -> impl Iterator<Item = Keymap> {
    let global_maps = get_global_keymaps().into_iter();
    let list_maps = get_entries_list_keymaps().into_iter();
    let editor_maps = get_editor_mode_keymaps().into_iter();

    global_maps.chain(list_maps).chain(editor_maps)
}

impl KeybindingsTable for GlobalBindings {
    fn get_state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    fn get_bindings_map(&self) -> &BTreeMap<UICommand, Vec<Input>> {
        &self.bindings_map
    }

    fn get_title(&self) -> &str {
        "Global Keybindings"
    }
}
