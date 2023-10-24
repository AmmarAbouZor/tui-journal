use std::path::PathBuf;

use backend::DataProvider;

use self::{
    editor::{Editor, EditorMode},
    entries_list::EntriesList,
    entry_popup::{EntryPopup, EntryPopupInputReturn},
    export_popup::ExportPopup,
    filter_popup::FilterPopup,
    footer::render_footer,
    fuzz_find::FuzzFindPopup,
    help_popup::{HelpInputInputReturn, HelpPopup},
    msg_box::{MsgBox, MsgBoxActions, MsgBoxType},
};

use super::{
    keymap::{
        get_editor_mode_keymaps, get_entries_list_keymaps, get_global_keymaps,
        get_multi_select_keymaps, Input, Keymap,
    },
    runner::HandleInputReturnType,
    App,
};
use anyhow::Result;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Color,
    Frame,
};

mod commands;
mod editor;
mod entries_list;
mod entry_popup;
mod export_popup;
mod filter_popup;
mod footer;
mod fuzz_find;
mod help_popup;
mod msg_box;
pub mod ui_functions;

pub use commands::UICommand;
pub use msg_box::MsgBoxResult;

pub const ACTIVE_CONTROL_COLOR: Color = Color::Reset;
pub const INACTIVE_CONTROL_COLOR: Color = Color::Rgb(170, 170, 200);
pub const EDITOR_MODE_COLOR: Color = Color::LightGreen;
pub const INVALID_CONTROL_COLOR: Color = Color::LightRed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    EntriesList,
    EntryContentTxt,
}

pub enum Popup<'a> {
    Help(Box<HelpPopup>),
    Entry(Box<EntryPopup<'a>>),
    MsgBox(Box<MsgBox>),
    Export(Box<ExportPopup<'a>>),
    Filter(Box<FilterPopup<'a>>),
    FuzzFind(Box<FuzzFindPopup<'a>>),
}

pub struct UIComponents<'a> {
    global_keymaps: Vec<Keymap>,
    entries_list_keymaps: Vec<Keymap>,
    editor_keymaps: Vec<Keymap>,
    multi_select_keymaps: Vec<Keymap>,
    entries_list: EntriesList,
    editor: Editor<'a>,
    popup_stack: Vec<Popup<'a>>,
    pub active_control: ControlType,
    pending_command: Option<UICommand>,
}

impl<'a, 'b> UIComponents<'a> {
    pub fn new() -> Self {
        let global_keymaps = get_global_keymaps();
        let entries_list_keymaps = get_entries_list_keymaps();
        let editor_keymaps = get_editor_mode_keymaps();
        let multi_select_keymaps = get_multi_select_keymaps();
        let mut entries_list = EntriesList::new();
        let editor = Editor::new();

        let active_control = ControlType::EntriesList;
        entries_list.set_active(true);

        Self {
            global_keymaps,
            entries_list_keymaps,
            editor_keymaps,
            multi_select_keymaps,
            entries_list,
            editor,
            popup_stack: Vec::new(),
            active_control,
            pending_command: None,
        }
    }

    pub fn has_popup(&self) -> bool {
        !self.popup_stack.is_empty()
    }

    pub fn set_current_entry<D: DataProvider>(&mut self, entry_id: Option<u32>, app: &mut App<D>) {
        app.current_entry_id = entry_id;
        if let Some(id) = entry_id {
            let entry_index = app.get_active_entries().position(|entry| entry.id == id);
            self.entries_list.state.select(entry_index);
        }

        self.editor.set_current_entry(entry_id, app);
    }

    pub fn render_ui<D>(&mut self, f: &mut Frame, app: &'b App<D>)
    where
        D: DataProvider,
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
            .split(f.size());

        render_footer(f, chunks[1], self, app);

        let entries_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(chunks[0]);

        self.entries_list
            .render_widget(f, entries_chunks[0], app, &self.entries_list_keymaps);
        self.editor.render_widget(f, entries_chunks[1]);

        self.render_popup(f);
    }

    pub fn render_popup(&mut self, f: &mut Frame) {
        if let Some(popup) = self.popup_stack.last_mut() {
            match popup {
                Popup::Help(help_popup) => help_popup.render_widget(f, f.size()),
                Popup::Entry(entry_popup) => entry_popup.render_widget(f, f.size()),
                Popup::MsgBox(msg_box) => msg_box.render_widget(f, f.size()),
                Popup::Export(export_popup) => export_popup.render_widget(f, f.size()),
                Popup::Filter(filter_popup) => filter_popup.render_widget(f, f.size()),
                Popup::FuzzFind(fuzz_find) => fuzz_find.render_widget(f, f.size()),
            }
        }
    }

    pub async fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        if self.has_popup() {
            return self.handle_popup_input(input, app).await;
        }

        if self.editor.is_insert_mode() {
            if let Some(key) = self.editor_keymaps.iter().find(|c| &c.key == input) {
                return key.command.clone().execute(self, app).await;
            }
            return self.editor.handle_input(input, app);
        }

        if self.entries_list.multi_select_mode {
            if let Some(key) = self.multi_select_keymaps.iter().find(|c| &c.key == input) {
                return key.command.to_owned().execute(self, app).await;
            }
            return Ok(HandleInputReturnType::Handled);
        }

        if let Some(cmd) = self
            .global_keymaps
            .iter()
            .find(|keymap| keymap.key == *input)
            .map(|keymap| keymap.command)
        {
            cmd.execute(self, app).await
        } else {
            match self.active_control {
                ControlType::EntriesList => {
                    if let Some(key) = self.entries_list_keymaps.iter().find(|c| &c.key == input) {
                        key.command.clone().execute(self, app).await
                    } else {
                        Ok(HandleInputReturnType::NotFound)
                    }
                }
                ControlType::EntryContentTxt => {
                    if let Some(key) = self.editor_keymaps.iter().find(|c| &c.key == input) {
                        key.command.clone().execute(self, app).await
                    } else {
                        self.editor.handle_input(input, app)
                    }
                }
            }
        }
    }

    async fn handle_popup_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> Result<HandleInputReturnType> {
        if let Some(popup) = self.popup_stack.last_mut() {
            match popup {
                Popup::Help(help_popup) => {
                    if help_popup.handle_input(input) == HelpInputInputReturn::Close {
                        self.popup_stack.pop().expect("popup stack isn't empty");
                    }
                }
                Popup::Entry(entry_popup) => {
                    let close_popup = match entry_popup.handle_input(input, app).await? {
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
                        self.popup_stack.pop().expect("popup stack isn't empty");
                    }
                }
                Popup::MsgBox(msg_box) => match msg_box.handle_input(input) {
                    msg_box::MsgBoxInputResult::Keep => {}
                    msg_box::MsgBoxInputResult::Close(msg_box_result) => {
                        self.popup_stack.pop().expect("popup stack isn't empty");
                        if let Some(cmd) = self.pending_command.take() {
                            return cmd.continue_executing(self, app, msg_box_result).await;
                        }
                    }
                },
                Popup::Export(export_popup) => {
                    match export_popup.handle_input(input) {
                        export_popup::ExportPopupInputReturn::KeepPopup => {}
                        export_popup::ExportPopupInputReturn::Cancel => {
                            self.popup_stack.pop().expect("popup stack isn't empty");
                        }
                        export_popup::ExportPopupInputReturn::Export(path, entry_id) => {
                            self.handle_export_popup_return(path, entry_id, app).await;
                        }
                    };
                }
                Popup::Filter(filter_popup) => match filter_popup.handle_input(input) {
                    filter_popup::FilterPopupReturn::KeepPopup => {}
                    filter_popup::FilterPopupReturn::Cancel => {
                        self.popup_stack.pop().expect("popup stack isn't empty");
                    }
                    filter_popup::FilterPopupReturn::Apply(filter) => {
                        app.apply_filter(filter);
                        self.popup_stack.pop().expect("popup stack isn't empty");

                        // This fixes the bug: Entry will not be highlighted when the result of the filter is one entry only
                        if app.get_active_entries().count() == 1 {
                            let entry_id = app.get_active_entries().next().map(|entry| entry.id);
                            self.set_current_entry(entry_id, app);
                        }
                    }
                },
                Popup::FuzzFind(fuzz_find) => match fuzz_find.handle_input(input) {
                    fuzz_find::FuzzFindReturn::Close => {
                        self.popup_stack.pop().expect("popup stack isn't empty");
                    }
                    fuzz_find::FuzzFindReturn::SelectEntry(entry_id) => {
                        if entry_id.is_some() {
                            self.set_current_entry(entry_id, app);
                        }
                    }
                },
            }
            Ok(HandleInputReturnType::Handled)
        } else {
            Ok(HandleInputReturnType::NotFound)
        }
    }

    #[inline]
    async fn handle_export_popup_return<D: DataProvider>(
        &mut self,
        path: PathBuf,
        entry_id: Option<u32>,
        app: &mut App<D>,
    ) {
        let (result, confirmation_msg) = if self.entries_list.multi_select_mode {
            let result = app.export_entries(path.clone()).await;
            let msg = format!("Journal(s)  exported to file {}", path.display());

            (result, msg)
        } else {
            let entry_id = entry_id.expect("entry id must have a value in normal mode");
            let result = app.export_entry_content(entry_id, path.clone()).await;
            let msg = format!("Journal content exported to file {}", path.display());

            (result, msg)
        };

        match result {
            Ok(_) => {
                self.popup_stack.pop().expect("popup stack isn't empty");

                if app.settings.export.show_confirmation {
                    self.show_msg_box(MsgBoxType::Info(confirmation_msg), MsgBoxActions::Ok, None);
                }
            }
            Err(err) => {
                self.show_err_msg(format!("Error while exporting journal(s). Err: {err}",));
            }
        };
    }

    fn set_control_is_active(&mut self, control: ControlType, is_active: bool) {
        match control {
            ControlType::EntriesList => self.entries_list.set_active(is_active),
            ControlType::EntryContentTxt => self.editor.set_active(is_active),
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

    fn start_edit_current_entry(&mut self) -> Result<HandleInputReturnType> {
        if self.entries_list.state.selected().is_none() {
            return Ok(HandleInputReturnType::Handled);
        }

        self.change_active_control(ControlType::EntryContentTxt);

        assert!(!self.editor.is_insert_mode());
        self.editor.mode = EditorMode::Insert;
        Ok(HandleInputReturnType::Handled)
    }

    pub fn show_msg_box(
        &mut self,
        msg: MsgBoxType,
        msg_actions: MsgBoxActions,
        pending_cmd: Option<UICommand>,
    ) {
        self.pending_command = pending_cmd;
        let msg_box = MsgBox::new(msg, msg_actions);

        self.popup_stack.push(Popup::MsgBox(Box::new(msg_box)));
    }

    pub fn show_unsaved_msg_box(&mut self, pending_cmd: Option<UICommand>) {
        self.pending_command = pending_cmd;
        let msg =
            MsgBoxType::Question("Do you want to save the changes on the current journal?".into());
        let msg_actions = MsgBoxActions::YesNoCancel;
        let msg_box = MsgBox::new(msg, msg_actions);

        self.popup_stack.push(Popup::MsgBox(Box::new(msg_box)));
    }

    #[inline]
    pub fn has_unsaved(&self) -> bool {
        self.editor.has_unsaved()
    }

    pub fn show_err_msg(&mut self, err_txt: String) {
        self.show_msg_box(MsgBoxType::Error(err_txt), MsgBoxActions::Ok, None);
    }

    pub fn update_current_entry<D: DataProvider>(&mut self, app: &mut App<D>) {
        if app.get_current_entry().is_none() {
            let first_entry = app.get_active_entries().next().map(|entry| entry.id);
            self.set_current_entry(first_entry, app);
        }
    }
}
