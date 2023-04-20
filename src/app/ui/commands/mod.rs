use std::fmt::Debug;

use crate::data::DataProvider;

use super::{App, HandleInputReturnType, MsgBoxResult, UIComponents};

use editor_cmd::*;
use entries_list_cmd::*;
use global_cmd::*;

mod editor_cmd;
mod entries_list_cmd;
mod global_cmd;

type CmdResult = anyhow::Result<HandleInputReturnType>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UICommand {
    Quit,
    ShowHelp,
    CycleFocusedControlForward,
    CycleFocusedControlBack,
    SelectedNextEntry,
    SelectedPrevEntry,
    CreateEntry,
    EditCurrentEntry,
    DeleteCurrentEntry,
    StartEditEntryContent,
    FinishEditEntryContent,
    SaveEntryContent,
    DiscardChangesEntryContent,
    ReloadAll,
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
}

impl CommandInfo {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
        }
    }
}

impl UICommand {
    pub fn get_info(&self) -> CommandInfo {
        match self {
            UICommand::Quit => CommandInfo::new("Exit", "Exit the program"),
            UICommand::ShowHelp => CommandInfo::new("Show help", "Show keybindings overview"),
            UICommand::CycleFocusedControlForward => {
                CommandInfo::new("Cycle focus forward", "Move focus to the next control")
            }
            UICommand::CycleFocusedControlBack => {
                CommandInfo::new("Cycle focus backward", "Move focus to the previous control")
            }
            UICommand::SelectedNextEntry => {
                CommandInfo::new("Select next entry", "Select next entry in the entry list")
            }
            UICommand::SelectedPrevEntry => CommandInfo::new(
                "Select previous entry",
                "Select previous entry in the entry list",
            ),
            UICommand::CreateEntry => CommandInfo::new(
                "Create new entry",
                "Opens dialog to add a new journal entry",
            ),
            UICommand::EditCurrentEntry => {
                CommandInfo::new("Edit current entry", "Edit current journal entry if any")
            }
            UICommand::DeleteCurrentEntry => CommandInfo::new(
                "Delete current entry",
                "Delete current journal entry if any",
            ),
            UICommand::StartEditEntryContent => CommandInfo::new(
                "Edit current entry content",
                "Edit current journal entry content if any",
            ),
            UICommand::FinishEditEntryContent => CommandInfo::new(
                "End editing current entry",
                "End editing current journal entry, and return focus to entries list",
            ),
            UICommand::SaveEntryContent => {
                CommandInfo::new("Save", "Save changes on journal content")
            }
            UICommand::DiscardChangesEntryContent => {
                CommandInfo::new("Discard changes", "Discard changes on journal content")
            }
            UICommand::ReloadAll => CommandInfo::new(
                "Reload all",
                "Reload all entries, discarding unsaved changes",
            ),
        }
    }

    pub fn execute<D: DataProvider>(
        &self,
        ui_components: &mut UIComponents,
        app: &mut App<D>,
    ) -> CmdResult {
        match self {
            UICommand::Quit => exec_quit(ui_components),
            UICommand::ShowHelp => exec_show_help(ui_components),
            UICommand::CycleFocusedControlForward => exec_cycle_forward(ui_components),
            UICommand::CycleFocusedControlBack => exec_cycle_backward(ui_components),
            UICommand::SelectedNextEntry => exec_select_next_entry(ui_components, app),
            UICommand::SelectedPrevEntry => exec_select_prev_entry(ui_components, app),
            UICommand::CreateEntry => exec_create_entry(ui_components),
            UICommand::EditCurrentEntry => exec_edit_current_entry(ui_components, app),
            UICommand::DeleteCurrentEntry => todo!(),
            UICommand::StartEditEntryContent => exec_start_edit_content(ui_components),
            UICommand::FinishEditEntryContent => exec_finish_editing(ui_components),
            UICommand::SaveEntryContent => exec_save_entry_content(ui_components, app),
            UICommand::DiscardChangesEntryContent => todo!(),
            UICommand::ReloadAll => exec_reload_all(ui_components, app),
        }
    }

    pub fn continue_executing<D: DataProvider>(
        &mut self,
        ui_components: &mut UIComponents,
        app: &mut App<D>,
        msg_box_result: MsgBoxResult,
    ) -> CmdResult {
        let not_implemented = || unreachable!("continue exec isn't implemented for {:?}", self);
        match self {
            UICommand::Quit => continue_quit(ui_components, app, msg_box_result),
            UICommand::ShowHelp => not_implemented(),
            UICommand::CycleFocusedControlForward => not_implemented(),
            UICommand::CycleFocusedControlBack => not_implemented(),
            UICommand::SelectedNextEntry => {
                continue_select_next_entry(ui_components, app, msg_box_result)
            }
            UICommand::SelectedPrevEntry => {
                continue_select_prev_entry(ui_components, app, msg_box_result)
            }
            UICommand::CreateEntry => continue_create_entry(ui_components, app, msg_box_result),
            UICommand::EditCurrentEntry => {
                continue_edit_current_entry(ui_components, app, msg_box_result)
            }
            UICommand::DeleteCurrentEntry => todo!(),
            UICommand::StartEditEntryContent => not_implemented(),
            UICommand::FinishEditEntryContent => not_implemented(),
            UICommand::SaveEntryContent => not_implemented(),
            UICommand::DiscardChangesEntryContent => todo!(),
            UICommand::ReloadAll => continue_reload_all(ui_components, app),
        }
    }
}
