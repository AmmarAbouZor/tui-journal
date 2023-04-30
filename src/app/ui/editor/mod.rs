use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
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

pub struct Editor<'a> {
    text_area: TextArea<'a>,
    is_active: bool,
    is_dirty: bool,
    has_unsaved: bool,
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

impl<'a> Editor<'a> {
    pub fn new() -> Editor<'a> {
        let text_area = TextArea::default();

        Editor {
            text_area,
            is_active: false,
            is_dirty: false,
            has_unsaved: false,
        }
    }

    pub fn set_current_entry<D: DataProvider>(&mut self, entry_id: Option<u32>, app: &App<D>) {
        let text_area = match entry_id {
            Some(id) => {
                if let Some(entry) = app.get_entry(id) {
                    self.is_dirty = false;
                    let lines = entry.content.lines().map(|line| line.to_owned()).collect();
                    let mut text_area = TextArea::new(lines);
                    text_area.move_cursor(tui_textarea::CursorMove::Bottom);
                    text_area.move_cursor(tui_textarea::CursorMove::End);
                    text_area
                } else {
                    TextArea::default()
                }
            }
            None => TextArea::default(),
        };

        self.text_area = text_area;

        self.refresh_has_unsaved(app);
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        is_edit_mode: bool,
        app: &App<D>,
    ) -> anyhow::Result<HandleInputReturnType> {
        if is_edit_mode {
            // give the input to the editor
            let key_event = KeyEvent::from(input);
            if self.text_area.input(key_event) {
                self.is_dirty = true;
                self.refresh_has_unsaved(app);
            }
        } else {
            let has_control = input.modifiers.contains(KeyModifiers::CONTROL);
            let has_alt = input.modifiers.contains(KeyModifiers::ALT);
            let is_navigation = match input.key_code {
                KeyCode::Left
                | KeyCode::Right
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown => true,
                KeyCode::Char('p') if has_control || has_alt => true,
                KeyCode::Char('n') if has_control || has_alt => true,
                KeyCode::Char('f') if has_control || has_alt => true,
                KeyCode::Char('b') if has_control || has_alt => true,
                KeyCode::Char('e') if has_control || has_alt => true,
                KeyCode::Char('a') if has_control || has_alt => true,
                KeyCode::Char('v') if has_control || has_alt => true,
                _ => false,
            };
            if is_navigation {
                let key_event = KeyEvent::from(input);
                self.text_area.input(key_event);
            }
        }
        Ok(HandleInputReturnType::Handled)
    }

    pub fn render_widget<B>(&mut self, frame: &mut Frame<B>, area: Rect, is_editor_mode: bool)
    where
        B: Backend,
    {
        let mut title = "Journal Content".to_owned();
        if self.has_unsaved {
            title.push_str(" *");
        }

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
                .title(title),
        );

        self.text_area
            .set_cursor_style(match (is_editor_mode, self.is_active) {
                (_, false) => Style::default(),
                (true, true) => Style::default().bg(EDITOR_MODE_COLOR).fg(Color::Black),
                (false, true) => Style::default().bg(Color::White).fg(Color::Black),
            });

        self.text_area.set_cursor_line_style(Style::default());

        frame.render_widget(self.text_area.widget(), area);
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn get_content(&self) -> String {
        let lines = self.text_area.lines().to_vec();

        lines.join("\n")
    }

    pub fn has_unsaved(&self) -> bool {
        self.has_unsaved
    }

    pub fn refresh_has_unsaved<D: DataProvider>(&mut self, app: &App<D>) {
        self.has_unsaved = match self.is_dirty {
            true => {
                if let Some(entry) = app.get_current_entry() {
                    self.is_dirty && entry.content != self.get_content()
                } else {
                    false
                }
            }
            false => false,
        }
    }
}
