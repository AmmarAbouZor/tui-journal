use anyhow::Result;
use std::fmt::Debug;

use super::App;

pub trait Command: Debug {
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn execute(&self, app: &mut App) -> Result<()>;
    fn can_exec(&self, app: &mut App) -> bool;
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

    fn execute(&self, app: &mut App) -> Result<()> {
        todo!()
    }

    fn can_exec(&self, app: &mut App) -> bool {
        todo!()
    }
}

#[derive(Debug, Default)]
pub(crate) struct DeleteCurrentEntry {}

impl Command for DeleteCurrentEntry {
    fn get_name(&self) -> &str {
        "Delete current entry"
    }

    fn get_description(&self) -> &str {
        "Delete current journal entry if any"
    }

    fn execute(&self, app: &mut App) -> Result<()> {
        todo!()
    }

    fn can_exec(&self, app: &mut App) -> bool {
        todo!()
    }
}
