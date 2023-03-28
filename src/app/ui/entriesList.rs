use crate::app::commands::UICommand;
use crate::app::keymap::Keymap;
use crate::data::DataProvider;

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

    fn handle_input<D: DataProvider>(
        &self,
        input: &crate::app::keymap::Input,
        app: &mut crate::app::App<D>,
    ) -> anyhow::Result<bool> {
        if let Some(key) = self.keymaps.iter().find(|&c| &c.key == input) {
            match key.command {
                UICommand::CreateEntry => {}
                UICommand::DeleteCurrentEntry => {}
                UICommand::StartEditCurrentEntry => {}
                _ => unreachable!("{:?} is not implemented for entries list", key.command),
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
