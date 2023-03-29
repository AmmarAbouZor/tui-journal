use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum UICommand {
    CreateEntry,
    DeleteCurrentEntry,
    StartEditCurrentEntry,
    SaveEntry,
    DiscardEntry,
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
    fn get_info(&self) -> CommandInfo {
        match self {
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
            UICommand::SaveEntry => CommandInfo::new("Save", "Save changes on journal"),
            UICommand::DiscardEntry => {
                CommandInfo::new("Discard changes", "Discard changes on journal")
            }
            UICommand::ReloadAll => CommandInfo::new(
                "Reload all",
                "Reload all entries, discarding unsaved changes",
            ),
        }
    }
}
