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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestBindingsTable {
        state: TableState,
        bindings: BTreeMap<UICommand, Vec<Input>>,
    }

    impl TestBindingsTable {
        fn new() -> Self {
            Self {
                state: TableState::default(),
                bindings: BTreeMap::from([
                    (
                        UICommand::Quit,
                        vec![Input::new(
                            crossterm::event::KeyCode::Char('q'),
                            crossterm::event::KeyModifiers::NONE,
                        )],
                    ),
                    (
                        UICommand::Undo,
                        vec![Input::new(
                            crossterm::event::KeyCode::Char('u'),
                            crossterm::event::KeyModifiers::NONE,
                        )],
                    ),
                    (
                        UICommand::Redo,
                        vec![Input::new(
                            crossterm::event::KeyCode::Char('U'),
                            crossterm::event::KeyModifiers::SHIFT,
                        )],
                    ),
                ]),
            }
        }
    }

    impl KeybindingsTable for TestBindingsTable {
        fn get_state_mut(&mut self) -> &mut TableState {
            &mut self.state
        }

        fn get_bindings_map(&self) -> &BTreeMap<UICommand, Vec<Input>> {
            &self.bindings
        }

        fn get_title(&self) -> &str {
            "Test"
        }
    }

    #[test]
    fn next_wraps_to_start() {
        let mut table = TestBindingsTable::new();
        table.state.select(Some(2));

        table.select_next();

        assert_eq!(table.state.selected(), Some(0));
    }

    #[test]
    fn previous_wraps_to_end() {
        let mut table = TestBindingsTable::new();
        table.state.select(Some(0));

        table.select_previous();

        assert_eq!(table.state.selected(), Some(2));
    }

    #[test]
    fn none_selection_picks_edges() {
        let mut table = TestBindingsTable::new();

        table.select_next();
        assert_eq!(table.state.selected(), Some(0));

        table.state.select(None);
        table.select_previous();
        assert_eq!(table.state.selected(), Some(2));
    }
}
