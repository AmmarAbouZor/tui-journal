use crate::data::DataProvider;

use super::{
    keymap::{Input, Keymap},
    runner::HandleInputReturnType,
    App,
};
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

mod entries_list;

pub use entries_list::EntriesList;

pub enum ControlType {
    EntriesList,
    EntryNameTxt,
    EntryContentTxt,
    HelpPopup,
}

pub trait UIComponent<'a> {
    fn handle_input<D: DataProvider>(
        &self,
        input: &Input,
        app: &'a mut App<D>,
    ) -> Result<HandleInputReturnType>;
    fn get_keymaps(&self) -> &[Keymap];
    fn get_type(&self) -> ControlType;
    fn render_widget<B: Backend, D: DataProvider>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        app: &'a App<D>,
    );
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

        self.entries_list.render_widget(f, chunks[0], app);
    }

    fn get_active_control(&mut self) -> &mut impl UIComponent {
        &mut self.entries_list
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        let active_control = self.get_active_control();

        active_control.handle_input(input, app)
    }
}
