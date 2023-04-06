use crossterm::event::{KeyCode, KeyModifiers};
use tui::backend::Backend;

use crate::{
    app::{
        commands::UICommand,
        keymap::{Input, Keymap},
        runner::HandleInputReturnType,
        App,
    },
    data::DataProvider,
};

use super::{ControlType, UIComponent};

pub struct EntryContent {
    keymaps: Vec<Keymap>,
    pub is_edit_mode: bool,
}

impl EntryContent {
    pub fn new() -> EntryContent {
        let keymaps = vec![
            Keymap::new(
                Input::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                UICommand::SaveEntryContent,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                UICommand::DiscardChangesEntryContent,
            ),
            Keymap::new(
                Input::new(KeyCode::Esc, KeyModifiers::NONE),
                UICommand::FinishEditEntryContent,
            ),
        ];

        EntryContent {
            keymaps,
            is_edit_mode: false,
        }
    }
}

impl<'a> UIComponent<'a> for EntryContent {
    fn handle_input<D: DataProvider>(
        &self,
        input: &Input,
        app: &'a mut App<D>,
    ) -> anyhow::Result<HandleInputReturnType> {
        if let Some(key) = self.keymaps.iter().find(|c| &c.key == input) {
            match key.command {
                UICommand::SaveEntryContent => {}
                UICommand::DiscardChangesEntryContent => {}
                UICommand::FinishEditEntryContent => {}
                _ => unreachable!(
                    "{:?} is not implemented for entry content text box",
                    key.command
                ),
            }
            Ok(HandleInputReturnType::Handled)
        } else if self.is_edit_mode {
            // give the input to the editor
            Ok(HandleInputReturnType::Handled)
        } else {
            Ok(HandleInputReturnType::NotFound)
        }
    }

    fn get_keymaps(&self) -> &[Keymap] {
        &self.keymaps
    }

    fn get_type(&self) -> super::ControlType {
        ControlType::EntryContentTxt
    }

    fn render_widget<B, D>(
        &mut self,
        frame: &mut tui::Frame<B>,
        area: tui::layout::Rect,
        app: &'a crate::app::App<D>,
    ) where
        B: Backend,
        D: DataProvider,
    {
        todo!()
    }
}
