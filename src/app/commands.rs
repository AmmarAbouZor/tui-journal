use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum UICommand {
    CreateEntry,
    DeleteCurrentEntry,
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
        }
    }
}
