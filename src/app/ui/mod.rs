use crate::data::DataProvider;

use self::{
    editor::Editor,
    entries_list::EntriesList,
    entry_popup::{EntryPopup, EntryPopupInputReturn},
    footer::render_footer,
    help_popup::render_help_popup,
    msg_box::{MsgBox, MsgBoxActions, MsgBoxType},
};

use super::{
    commands::UICommand,
    keymap::{get_editor_keymaps, get_entries_list_keymaps, get_global_keymaps, Input, Keymap},
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

mod editor;
mod entries_list;
mod entry_popup;
mod footer;
mod help_popup;
mod msg_box;
mod ui_functions;

pub const ACTIVE_CONTROL_COLOR: Color = Color::Reset;
pub const INACTIVE_CONTROL_COLOR: Color = Color::Rgb(170, 170, 200);
pub const EDITOR_MODE_COLOR: Color = Color::LightYellow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    EntriesList,
    EntryContentTxt,
    HelpPopup,
    EntryPopup,
}

pub struct UIComponents<'a> {
    global_keymaps: Vec<Keymap>,
    entries_list_keymaps: Vec<Keymap>,
    editor_keymaps: Vec<Keymap>,
    entries_list: EntriesList,
    editor: Editor<'a>,
    entry_popup: EntryPopup<'a>,
    msg_box: Option<MsgBox>,
    pub active_control: ControlType,
    show_help_popup: bool,
    is_editor_mode: bool,
    show_entry_popup: bool,
}

impl<'a, 'b> UIComponents<'a> {
    pub fn new() -> Self {
        let global_keymaps = get_global_keymaps();
        let entries_list_keymaps = get_entries_list_keymaps();
        let editor_keymaps = get_editor_keymaps();
        let mut entries_list = EntriesList::new();
        let editor = Editor::new();
        let entry_popup = EntryPopup::new();

        let active_control = ControlType::EntriesList;
        entries_list.set_active(true);

        Self {
            global_keymaps,
            entries_list_keymaps,
            editor_keymaps,
            entries_list,
            editor,
            entry_popup,
            msg_box: None,
            active_control,
            show_help_popup: false,
            is_editor_mode: false,
            show_entry_popup: false,
        }
    }

    pub fn has_popup(&self) -> bool {
        self.show_help_popup || self.show_entry_popup || self.has_msg_box()
    }

    fn has_msg_box(&self) -> bool {
        self.msg_box.is_some()
    }

    pub fn set_current_entry<D: DataProvider>(&mut self, entry_id: Option<u32>, app: &mut App<D>) {
        if let Some(id) = entry_id {
            app.current_entry_id = entry_id;

            let entry_index = app.entries.iter().position(|entry| entry.id == id);
            self.entries_list.state.select(entry_index);

            self.editor.set_current_entry(entry_id, app);
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

        render_footer(f, chunks[1], self);

        let entries_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(chunks[0]);

        self.entries_list
            .render_widget(f, entries_chunks[0], &app.entries);
        self.editor
            .render_widget(f, entries_chunks[1], self.is_editor_mode);

        if self.show_help_popup {
            assert!(!self.show_entry_popup);
            render_help_popup(f, f.size(), self);
        }

        if self.show_entry_popup {
            assert!(!self.show_help_popup);
            self.entry_popup.render_widget(f, f.size());
        }

        if let Some(msg_box) = &mut self.msg_box {
            msg_box.render_widget(f, f.size());
        }
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        if self.has_popup() {
            return self.handle_popup_input(input, app);
        }

        if self.is_editor_mode {
            if let Some(key) = self.editor_keymaps.iter().find(|c| &c.key == input) {
                if key.command == UICommand::FinishEditEntryContent {
                    editor::execute_command(key.command, self, app)?;
                    return Ok(HandleInputReturnType::Handled);
                }
            }
            return self.editor.handle_input(input, true);
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
                    if let Some(key) = self.editor_keymaps.iter().find(|c| &c.key == input) {
                        editor::execute_command(key.command, self, app)?;
                        Ok(HandleInputReturnType::Handled)
                    } else {
                        self.editor.handle_input(input, self.is_editor_mode)
                    }
                }
                ControlType::HelpPopup | ControlType::EntryPopup => {
                    unreachable!("Popups must be handled at first, if they are active")
                }
            }
        }
    }

    fn handle_popup_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        if let Some(msg_box) = &self.msg_box {
            match msg_box.handle_input(input) {
                msg_box::MsgBoxInputResult::Keep => {}
                msg_box::MsgBoxInputResult::Close(_msg_box_result) => {
                    self.msg_box = None;
                    //TODO: check who is pending message box answer and let it handle the response
                }
            }
            return Ok(HandleInputReturnType::Handled);
        }

        match self.active_control {
            ControlType::EntriesList | ControlType::EntryContentTxt => {
                unreachable!("{:?} is not an popup control", self.active_control)
            }
            ControlType::HelpPopup => {
                // Close the help pop up on anykey
                self.show_help_popup = false;
                self.change_active_control(ControlType::EntriesList);
                Ok(HandleInputReturnType::Handled)
            }
            ControlType::EntryPopup => {
                //TODO: handle err case
                let close_popup = match self.entry_popup.handle_input(input, app)? {
                    EntryPopupInputReturn::Cancel => true,
                    EntryPopupInputReturn::KeepPupup => false,
                    EntryPopupInputReturn::AddEntry(entry_id) => {
                        self.set_current_entry(Some(entry_id), app);
                        true
                    }
                    EntryPopupInputReturn::UpdateCurrentEntry => {
                        self.set_current_entry(app.current_entry_id, app);
                        true
                    }
                };

                if close_popup {
                    self.show_entry_popup = false;
                    self.change_active_control(ControlType::EntriesList);
                }
                Ok(HandleInputReturnType::Handled)
            }
        }
    }

    fn set_control_is_active(&mut self, control: ControlType, is_active: bool) {
        match control {
            ControlType::EntriesList => self.entries_list.set_active(is_active),
            ControlType::EntryContentTxt => self.editor.set_active(is_active),
            ControlType::HelpPopup => {
                // HelpPopup doesn't have active logic
            }
            ControlType::EntryPopup => self.entry_popup.set_active(is_active),
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
                    ControlType::EntryPopup => todo!(),
                };

                self.change_active_control(next_control);

                Ok(HandleInputReturnType::Handled)
            }
            UICommand::CycleFocusedControlBack => {
                let prev_control = match self.active_control {
                    ControlType::EntriesList => ControlType::EntryContentTxt,
                    ControlType::EntryContentTxt => ControlType::EntriesList,
                    ControlType::HelpPopup => ControlType::EntriesList,
                    ControlType::EntryPopup => todo!(),
                };

                self.change_active_control(prev_control);

                Ok(HandleInputReturnType::Handled)
            }
            UICommand::StartEditEntryContent => self.start_edit_current_entry(),
            UICommand::ReloadAll => {
                //TODO: Remove test code and implement ReloadAll
                let test_msg_box = MsgBox::new(
                    MsgBoxType::Question("Message very very long text to check the wrapping very very long text to check the wrapping".into()),
                    MsgBoxActions::YesNoCancel,
                );

                self.msg_box = Some(test_msg_box);

                Ok(HandleInputReturnType::Handled)
            }
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
        let editor_maps = self.editor_keymaps.iter();

        global_maps.chain(list_maps).chain(editor_maps)
    }
}
