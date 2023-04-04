use crate::data::DataProvider;

use super::{
    keymap::{Input, Keymap},
    runner::HandleInputReturnType,
    App,
};
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::Widget,
    Frame,
};

mod entriesList;

pub use entriesList::EntriesList;

pub enum ControlType {
    EntriesList,
    EntryNameTxt,
    EntryContentTxt,
    HelpPopup,
}

pub trait UIComponent<'a, W>
where
    W: Widget,
{
    fn handle_input<D: DataProvider>(
        &self,
        input: &Input,
        app: &'a mut App<D>,
    ) -> Result<HandleInputReturnType>;
    fn get_keymaps(&self) -> &[Keymap];
    fn get_type(&self) -> ControlType;
    fn get_widget<D: DataProvider>(&self, app: &'a App<D>) -> W;
}

pub struct UIComponents {
    pub entries_list: EntriesList,
    pub active_control: ControlType,
}

impl<'a> UIComponents {
    pub fn new() -> Self {
        let entries_list = EntriesList::new();
        let active_control = ControlType::EntriesList;
        Self {
            entries_list,
            active_control,
        }
    }

    pub fn set_current_entry<D: DataProvider>(&mut self, entry_id: Option<u32>, app: &App<D>) {
        if let Some(id) = entry_id {
            let entry_index = app.entries.iter().position(|entry| entry.id == id);

            self.entries_list.state.select(entry_index);
        }
    }

    pub fn get_current_entry_id<D: DataProvider>(&self, app: &App<D>) -> Option<u32> {
        if let Some(index) = self.entries_list.state.selected() {
            app.entries.get(index).and_then(|entry| Some(entry.id))
        } else {
            None
        }
    }

    pub fn draw_ui<D, B>(&mut self, f: &mut Frame<B>, app: &'a App<D>)
    where
        D: DataProvider,
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
            .split(f.size());

        let entries_widget = self.entries_list.get_widget(app);
        let list_state = self.entries_list.get_state_mut();

        f.render_stateful_widget(entries_widget, chunks[0], list_state);
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        todo!()
    }
}
