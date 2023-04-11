use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UICommand {
    Quit,
    ShowHelp,
    CycleFocusedControlForward,
    CycleFocusedControlBack,
    SelectedNextEntry,
    SelectedPrevEntry,
    CreateEntry,
    DeleteCurrentEntry,
    StartEditCurrentEntry,
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
            UICommand::DeleteCurrentEntry => CommandInfo::new(
                "Delete current entry",
                "Delete current journal entry if any",
            ),
            UICommand::StartEditCurrentEntry => {
                CommandInfo::new("Edit current entry", "Edit current journal entry if any")
            }
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
}
