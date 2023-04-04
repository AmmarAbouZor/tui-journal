use crossterm::event::{KeyCode, KeyModifiers};

use crate::data::{DataProvider, Entry};

use self::{
    commands::UICommand,
    keymap::{Input, Keymap},
    runner::HandleInputReturnType,
};

pub use runner::run;
pub use ui::UIComponents;

mod commands;
mod keymap;
mod runner;
mod ui;

pub struct App<D>
where
    D: DataProvider,
{
    pub data_provide: D,
    pub entries: Vec<Entry>,
    pub global_keymaps: Vec<Keymap>,
}

impl<D> App<D>
where
    D: DataProvider,
{
    pub fn new(data_provide: D) -> Self {
        let entries = Vec::new();
        let global_keymaps = vec![
            Keymap::new(
                Input::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
                UICommand::Quit,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('?'), KeyModifiers::SHIFT),
                UICommand::ShowHelp,
            ),
        ];
        Self {
            data_provide,
            entries,
            global_keymaps,
        }
    }

    pub fn load_entries(&mut self) -> anyhow::Result<()> {
        self.entries = self.data_provide.load_all_entries()?;

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(())
    }

    pub fn handle_input(&self, input: &Input) -> HandleInputReturnType {
        if let Some(cmd) = self
            .global_keymaps
            .iter()
            .find(|keymap| keymap.key == *input)
            .and_then(|keymap| Some(keymap.command))
        {
            match cmd {
                UICommand::Quit => HandleInputReturnType::ExitApp,
                UICommand::ShowHelp => {
                    // TODO: show help

                    HandleInputReturnType::Handled
                }
                _ => unreachable!("command '{:?}' is not implemented in global keymaps", cmd),
            }
        } else {
            HandleInputReturnType::NotFound
        }
    }
}
