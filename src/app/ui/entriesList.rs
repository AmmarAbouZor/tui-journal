use crate::app::commands::UICommand;
use crate::app::keymap::Keymap;

use super::UIComponent;

pub struct EntriesList {
    keymaps: Vec<Keymap>,
}

impl UIComponent for EntriesList {
    fn get_keymaps(&self) -> &[crate::app::keymap::Keymap] {
        &self.keymaps
    }

    fn get_type(&self) -> super::ControlType {
        super::ControlType::EntriesList
    }

    fn handle_input(
        &self,
        input: &crate::app::keymap::Input,
        app: &mut crate::app::App,
    ) -> anyhow::Result<bool> {
        if let Some(key) = self.keymaps.iter().find(|&c| &c.key == input) {
            match key.command {
                UICommand::CreateEntry => {}
                UICommand::DeleteCurrentEntry => {}
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
