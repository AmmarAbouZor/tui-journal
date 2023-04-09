use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use tui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    app::{
        commands::UICommand,
        keymap::{Input, Keymap},
        runner::HandleInputReturnType,
        App,
    },
    data::DataProvider,
};
use tui_textarea::TextArea;

use super::{ControlType, UIComponent, ACTIVE_CONTROL_COLOR};

pub struct EntryContent<'a> {
    keymaps: Vec<Keymap>,
    text_area: TextArea<'a>,
    // edit mode will be always on until I implement two modes for the editor
    is_active: bool,
    pub is_edit_mode: bool,
}

impl<'a> EntryContent<'a> {
    pub fn new() -> EntryContent<'a> {
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

        let text_area = TextArea::default();

        EntryContent {
            keymaps,
            text_area,
            is_active: false,
            is_edit_mode: true,
        }
    }
}

impl<'a> EntryContent<'a> {
    pub fn set_current_entry<D: DataProvider>(&mut self, entry_id: Option<u32>, app: &App<D>) {
        let text_area = match entry_id {
            Some(id) => {
                if let Some(entry) = app.get_entry(id) {
                    let lines = entry.content.lines().map(|line| line.to_owned()).collect();
                    TextArea::new(lines)
                } else {
                    TextArea::default()
                }
            }
            None => TextArea::default(),
        };

        self.text_area = text_area;
    }
}

impl From<&Input> for KeyEvent {
    fn from(value: &Input) -> Self {
        KeyEvent {
            code: value.key_code,
            modifiers: value.modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}

impl<'a, 'b> UIComponent<'b> for EntryContent<'a> {
    fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &'b mut App<D>,
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
            let key_event = KeyEvent::from(input);
            self.text_area.input(key_event);
            Ok(HandleInputReturnType::Handled)
        } else {
            //TODO: Implement vim normal modes shortcuts
            Ok(HandleInputReturnType::NotFound)
        }
    }

    fn get_keymaps(&self) -> &[Keymap] {
        &self.keymaps
    }

    fn get_type(&self) -> super::ControlType {
        ControlType::EntryContentTxt
    }

    fn render_widget<B, D>(&mut self, frame: &mut Frame<B>, area: Rect, app: &'b App<D>)
    where
        B: Backend,
        D: DataProvider,
    {
        self.text_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(match self.is_active {
                    true => Style::default().fg(ACTIVE_CONTROL_COLOR),
                    false => Style::default(),
                })
                .title("Journal content"),
        );

        frame.render_widget(self.text_area.widget(), area);
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}
