use crate::data::DataProvider;

use self::entry_content::EntryContent;

use super::{
    commands::UICommand,
    keymap::{Input, Keymap},
    runner::HandleInputReturnType,
    App,
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

mod entries_list;
mod entry_content;

pub use entries_list::EntriesList;

pub enum ControlType {
    EntriesList,
    EntryNameTxt,
    EntryContentTxt,
    HelpPopup,
}

pub trait UIComponent<'a> {
    fn handle_input<D: DataProvider>(
        &mut self,
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

pub struct UIComponents<'a> {
    pub global_keymaps: Vec<Keymap>,
    pub entries_list: EntriesList,
    pub entry_content: EntryContent<'a>,
    pub active_control: ControlType,
}

impl<'a, 'b> UIComponents<'a> {
    pub fn new() -> Self {
        let global_keymaps = vec![
            Keymap::new(
                Input::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
                UICommand::Quit,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('?'), KeyModifiers::SHIFT),
                UICommand::ShowHelp,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('j'), KeyModifiers::CONTROL),
                UICommand::CycleFocusedControlForward,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('k'), KeyModifiers::CONTROL),
                UICommand::CycleFocusedControlBack,
            ),
        ];
        let entries_list = EntriesList::new();
        let entry_content = EntryContent::new();
        let active_control = ControlType::EntriesList;
        Self {
            global_keymaps,
            entries_list,
            entry_content,
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

    pub fn draw_ui<D, B>(&mut self, f: &mut Frame<B>, app: &'b App<D>)
    where
        D: DataProvider,
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
            .split(f.size());

        self.entries_list.render_widget(f, chunks[0], app);
        self.entry_content.render_widget(f, chunks[1], app);
    }

    fn get_active_control(&mut self) -> &mut impl UIComponent {
        &mut self.entries_list
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        if let Some(cmd) = self
            .global_keymaps
            .iter()
            .find(|keymap| keymap.key == *input)
            .and_then(|keymap| Some(keymap.command))
        {
            match cmd {
                UICommand::Quit => Ok(HandleInputReturnType::ExitApp),
                UICommand::ShowHelp => {
                    // TODO: show help

                    Ok(HandleInputReturnType::Handled)
                }
                _ => unreachable!("command '{:?}' is not implemented in global keymaps", cmd),
            }
        } else {
            let active_control = self.get_active_control();

            active_control.handle_input(input, app)
        }
    }
}
