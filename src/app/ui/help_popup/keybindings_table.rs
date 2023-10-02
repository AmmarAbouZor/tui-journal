use std::collections::BTreeMap;

use ratatui::widgets::TableState;

use crate::app::{keymap::Input, ui::UICommand};

pub trait KeybindingsTable {
    fn get_state_mut(&mut self) -> &mut TableState;
    fn get_bindings_map(&self) -> &BTreeMap<UICommand, Vec<Input>>;
    fn get_title(&self) -> &str;

    fn select_next(&mut self) {
        let last_index = self.get_bindings_map().len() - 1;
        let state = self.get_state_mut();
        let new_row = state
            .selected()
            .map(|row| if row >= last_index { 0 } else { row + 1 })
            .unwrap_or(0);
        state.select(Some(new_row));
    }

    fn select_previous(&mut self) {
        let last_index = self.get_bindings_map().len() - 1;
        let state = self.get_state_mut();
        let new_row = state
            .selected()
            .map(|row| row.checked_sub(1).unwrap_or(last_index))
            .unwrap_or(last_index);

        state.select(Some(new_row));
    }
}
