use crate::data::DataProvider;

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
    fn handle_input<D: DataProvider>(&self, input: &Input, app: &mut App<D>) -> Result<bool>;
    fn get_keymaps(&self) -> &[Keymap];
    fn get_type(&self) -> ControlType;
}
