use super::{
    keymap::{Input, Keymap},
    App,
};
use anyhow::Result;

mod entriesList;

pub enum ControlType {
    EntriesList,
    EntryNameTxt,
    EntryContentTxt,
    HelpPopup,
}

pub trait UIComponent {
    fn handle_input(&self, input: &Input, app: &mut App) -> Result<bool>;
    fn get_keymaps(&self) -> &[Keymap];
    fn get_type(&self) -> ControlType;
}
