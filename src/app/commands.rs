use anyhow::Result;

use super::AppState;

pub trait Command {
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn execute(&self, app: &mut AppState) -> Result<()>;
    fn can_exec(&self, app: &mut AppState) -> bool;
}

#[derive(Debug, Default)]
pub(crate) struct CreateEntryCommand {}

impl Command for CreateEntryCommand {
    fn get_name(&self) -> &'static str {
        "Create new entry"
    }

    fn get_description(&self) -> &'static str {
        "Opens dialog to add a new journal entry"
    }

    fn execute(&self, app: &mut AppState) -> Result<()> {
        todo!()
    }

    fn can_exec(&self, app: &mut AppState) -> bool {
        todo!()
    }
}

#[derive(Debug, Default)]
pub(crate) struct DeleteCurrentEntry {}

impl Command for DeleteCurrentEntry {
    fn get_name(&self) -> &'static str {
        "Delete current entry"
    }

    fn get_description(&self) -> &'static str {
        "Delete current journal entry if any"
    }

    fn execute(&self, app: &mut AppState) -> Result<()> {
        todo!()
    }

    fn can_exec(&self, app: &mut AppState) -> bool {
        todo!()
    }
}
