use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    app::{keymap::Input, runner::HandleInputReturnType, App},
    data::DataProvider,
};
use tui_textarea::TextArea;

use super::ACTIVE_CONTROL_COLOR;
use super::EDITOR_MODE_COLOR;
use super::INACTIVE_CONTROL_COLOR;

mod command_actions;
pub(crate) use command_actions::execute_command;

pub struct EntryContent<'a> {
    text_area: TextArea<'a>,
    is_active: bool,
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

impl<'a, 'b> EntryContent<'a> {
    pub fn new() -> EntryContent<'a> {
        let text_area = TextArea::default();

        EntryContent {
            text_area,
            is_active: false,
        }
    }

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

    pub fn handle_input(
        &mut self,
        input: &Input,
        is_edit_mode: bool,
    ) -> anyhow::Result<HandleInputReturnType> {
        if is_edit_mode {
            // give the input to the editor
            let key_event = KeyEvent::from(input);
            self.text_area.input(key_event);
            Ok(HandleInputReturnType::Handled)
        } else {
            //TODO: Implement vim normal modes shortcuts
            Ok(HandleInputReturnType::NotFound)
        }
    }

    pub fn render_widget<B>(&mut self, frame: &mut Frame<B>, area: Rect, is_editor_mode: bool)
    where
        B: Backend,
    {
        self.text_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(match (self.is_active, is_editor_mode) {
                    (_, true) => Style::default()
                        .fg(EDITOR_MODE_COLOR)
                        .add_modifier(Modifier::BOLD),
                    (true, false) => Style::default()
                        .fg(ACTIVE_CONTROL_COLOR)
                        .add_modifier(Modifier::BOLD),
                    (false, false) => Style::default().fg(INACTIVE_CONTROL_COLOR),
                })
                .title("Journal content"),
        );

        self.text_area.set_cursor_style(match is_editor_mode {
            true => Style::default().bg(EDITOR_MODE_COLOR).fg(Color::Black),
            false => Style::default().bg(Color::White).fg(Color::Black),
        });

        frame.render_widget(self.text_area.widget(), area);
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}
