use crate::data::DataProvider;

use self::{entry_content::EntryContent, footer::render_footer, help_popup::render_help_popup};

use super::{
    commands::UICommand,
    keymap::{
        get_entries_list_keymaps, get_entry_content_keymaps, get_global_keymaps, Input, Keymap,
    },
    runner::HandleInputReturnType,
    App,
};
use anyhow::Result;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::Color,
    Frame,
};

mod entries_list;
mod entry_content;
mod footer;
mod help_popup;
mod ui_functions;

pub use entries_list::EntriesList;

pub const ACTIVE_CONTROL_COLOR: Color = Color::LightYellow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    EntriesList,
    EntryContentTxt,
    HelpPopup,
}

pub struct UIComponents<'a> {
    global_keymaps: Vec<Keymap>,
    entries_list_keymaps: Vec<Keymap>,
    entry_content_keymaps: Vec<Keymap>,
    pub entries_list: EntriesList,
    pub entry_content: EntryContent<'a>,
    pub active_control: ControlType,
    show_help_popup: bool,
    is_editor_mode: bool,
}

impl<'a, 'b> UIComponents<'a> {
    pub fn new() -> Self {
        let global_keymaps = get_global_keymaps();
        let entries_list_keymaps = get_entries_list_keymaps();
        let entry_content_keymaps = get_entry_content_keymaps();
        let mut entries_list = EntriesList::new();
        let entry_content = EntryContent::new();

        let active_control = ControlType::EntriesList;
        entries_list.set_active(true);

        Self {
            global_keymaps,
            entries_list_keymaps,
            entry_content_keymaps,
            entries_list,
            entry_content,
            active_control,
            show_help_popup: false,
            is_editor_mode: false,
        }
    }

    pub fn has_popup(&self) -> bool {
        self.show_help_popup
    }

    pub fn set_current_entry<D: DataProvider>(&mut self, entry_id: Option<u32>, app: &mut App<D>) {
        if let Some(id) = entry_id {
            app.current_entry_id = entry_id;

            let entry_index = app.entries.iter().position(|entry| entry.id == id);
            self.entries_list.state.select(entry_index);

            self.entry_content.set_current_entry(entry_id, app);
        }
    }

    pub fn render_ui<D, B>(&mut self, f: &mut Frame<B>, app: &'b App<D>)
    where
        D: DataProvider,
        B: Backend,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
            .split(f.size());

        render_footer(f, chunks[1], &self.global_keymaps);

        let entries_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(chunks[0]);

        self.entries_list
            .render_widget(f, entries_chunks[0], &app.entries);
        self.entry_content.render_widget(f, entries_chunks[1], app);

        if self.show_help_popup {
            render_help_popup(f, f.size(), self);
        }
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        if self.has_popup() {
            match self.active_control {
                ControlType::EntriesList | ControlType::EntryContentTxt => {
                    unreachable!("{:?} is not an popup control", self.active_control)
                }
                ControlType::HelpPopup => {
                    // Close the help pop up on anykey
                    self.show_help_popup = false;
                    self.active_control = ControlType::EntriesList;
                    self.set_control_is_active(ControlType::EntriesList, true);
                    return Ok(HandleInputReturnType::Handled);
                }
            }
        }

        if self.is_editor_mode {
            if let Some(key) = self.entry_content_keymaps.iter().find(|c| &c.key == input) {
                if key.command == UICommand::FinishEditEntryContent {
                    entry_content::execute_command(key.command, self, app)?;
                    return Ok(HandleInputReturnType::Handled);
                }
            }
            return self.entry_content.handle_input(input, true);
        }

        if let Some(cmd) = self
            .global_keymaps
            .iter()
            .find(|keymap| keymap.key == *input)
            .and_then(|keymap| Some(keymap.command))
        {
            self.execute_global_command(cmd, app)
        } else {
            match self.active_control {
                ControlType::EntriesList => {
                    if let Some(key) = self.entries_list_keymaps.iter().find(|c| &c.key == input) {
                        entries_list::execute_command(key.command, self, app)?;
                        Ok(HandleInputReturnType::Handled)
                    } else {
                        Ok(HandleInputReturnType::NotFound)
                    }
                }
                ControlType::EntryContentTxt => {
                    if let Some(key) = self.entry_content_keymaps.iter().find(|c| &c.key == input) {
                        entry_content::execute_command(key.command, self, app)?;
                        Ok(HandleInputReturnType::Handled)
                    } else {
                        self.entry_content.handle_input(input, self.is_editor_mode)
                    }
                }
                ControlType::HelpPopup => todo!(),
            }
        }
    }

    fn set_control_is_active(&mut self, control: ControlType, is_active: bool) {
        match control {
            ControlType::EntriesList => self.entries_list.set_active(is_active),
            ControlType::EntryContentTxt => self.entry_content.set_active(is_active),
            ControlType::HelpPopup => todo!(),
        }
    }

    pub fn change_active_control(&mut self, control: ControlType) {
        if self.active_control == control {
            return;
        }

        self.set_control_is_active(self.active_control, false);
        self.active_control = control;

        self.set_control_is_active(control, true);
    }

    fn execute_global_command<D: DataProvider>(
        &mut self,
        command: UICommand,
        _app: &mut App<D>,
    ) -> anyhow::Result<HandleInputReturnType> {
        match command {
            UICommand::Quit => Ok(HandleInputReturnType::ExitApp),
            UICommand::ShowHelp => {
                self.set_control_is_active(self.active_control, false);
                self.show_help_popup = true;
                self.active_control = ControlType::HelpPopup;

                Ok(HandleInputReturnType::Handled)
            }
            UICommand::CycleFocusedControlForward => {
                let next_control = match self.active_control {
                    ControlType::EntriesList => ControlType::EntryContentTxt,
                    ControlType::EntryContentTxt => ControlType::EntriesList,
                    ControlType::HelpPopup => ControlType::EntriesList,
                };

                self.change_active_control(next_control);

                Ok(HandleInputReturnType::Handled)
            }
            UICommand::CycleFocusedControlBack => {
                let prev_control = match self.active_control {
                    ControlType::EntriesList => ControlType::EntryContentTxt,
                    ControlType::EntryContentTxt => ControlType::EntriesList,
                    ControlType::HelpPopup => ControlType::EntriesList,
                };

                self.change_active_control(prev_control);

                Ok(HandleInputReturnType::Handled)
            }
            UICommand::StartEditCurrentEntry => self.start_edit_current_entry(),
            UICommand::ReloadAll => todo!(),
            _ => unreachable!(
                "command '{:?}' is not implemented in global keymaps",
                command
            ),
        }
    }

    fn start_edit_current_entry(&mut self) -> Result<HandleInputReturnType> {
        if self.entries_list.state.selected().is_none() {
            return Ok(HandleInputReturnType::Handled);
        }

        self.change_active_control(ControlType::EntryContentTxt);

        assert!(self.is_editor_mode == false);
        self.is_editor_mode = true;
        Ok(HandleInputReturnType::Handled)
    }

    fn get_all_keymaps(&self) -> impl Iterator<Item = &Keymap> {
        let global_maps = self.global_keymaps.iter();
        let list_maps = self.entries_list_keymaps.iter();
        let content_maps = self.entry_content_keymaps.iter();

        global_maps.chain(list_maps).chain(content_maps)
    }
}
