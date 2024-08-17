use anyhow::{anyhow, bail};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{
    layout::Rect,
    prelude::Margin,
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::{keymap::Input, runner::HandleInputReturnType, App};

use backend::DataProvider;
use tui_textarea::{CursorMove, Scrolling, TextArea};

use super::INACTIVE_CONTROL_COLOR;
use super::{commands::ClipboardOperation, EDITOR_MODE_COLOR};
use super::{ACTIVE_CONTROL_COLOR, VISUAL_MODE_COLOR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
}

pub struct Editor<'a> {
    text_area: TextArea<'a>,
    mode: EditorMode,
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
            mode: EditorMode::Normal,
            is_active: false,
            is_dirty: false,
            has_unsaved: false,
        }
    }

    #[inline]
    pub fn is_insert_mode(&self) -> bool {
        self.mode == EditorMode::Insert
    }

    #[inline]
    pub fn is_visual_mode(&self) -> bool {
        self.mode == EditorMode::Visual
    }

    #[inline]
    pub fn is_prioritized(&self) -> bool {
        matches!(self.mode, EditorMode::Insert | EditorMode::Visual)
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

    pub fn handle_input_prioritized<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &App<D>,
    ) -> anyhow::Result<HandleInputReturnType> {
        if self.is_insert_mode() {
            // We must handle clipboard operation separately if sync with system clipboard is
            // activated
            if app.settings.sync_os_clipboard {
                let has_ctrl = input.modifiers.contains(KeyModifiers::CONTROL);
                // Keymaps are taken from `text_area` source code
                let handeld = match input.key_code {
                    KeyCode::Char('x') if has_ctrl => {
                        self.exec_os_clipboard(ClipboardOperation::Cut)?;
                        true
                    }
                    KeyCode::Char('c') if has_ctrl => {
                        self.exec_os_clipboard(ClipboardOperation::Copy)?;
                        true
                    }
                    KeyCode::Char('y') if has_ctrl => {
                        self.exec_os_clipboard(ClipboardOperation::Paste)?;
                        true
                    }
                    _ => false,
                };

                if handeld {
                    return Ok(HandleInputReturnType::Handled);
                }
            }

            // give the input to the editor
            let key_event = KeyEvent::from(input);
            if self.text_area.input(key_event) {
                self.is_dirty = true;
                self.refresh_has_unsaved(app);
            }

            return Ok(HandleInputReturnType::Handled);
        }

        Ok(HandleInputReturnType::NotFound)
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &App<D>,
    ) -> anyhow::Result<HandleInputReturnType> {
        debug_assert!(!self.is_insert_mode());

        if app.get_current_entry().is_none() {
            return Ok(HandleInputReturnType::Handled);
        }

        let sync_os_clipboard = app.settings.sync_os_clipboard;

        if is_default_navigation(input) {
            let key_event = KeyEvent::from(input);
            self.text_area.input(key_event);
        } else if !self.is_visual_mode()
            || !self.handle_input_visual_only(input, sync_os_clipboard)?
        {
            self.handle_vim_motions(input, sync_os_clipboard)?;
        }

        // Check if the input led the editor to leave the visual mode and make the corresponding UI changes
        if !self.text_area.is_selecting() && self.is_visual_mode() {
            self.set_editor_mode(EditorMode::Normal);
        }

        self.is_dirty = true;
        self.refresh_has_unsaved(app);

        Ok(HandleInputReturnType::Handled)
    }

    /// Handles input specialized for visual mode only like cut and copy
    fn handle_input_visual_only(
        &mut self,
        input: &Input,
        sync_os_clipboard: bool,
    ) -> anyhow::Result<bool> {
        if !input.modifiers.is_empty() {
            return Ok(false);
        }

        match input.key_code {
            KeyCode::Char('d') => {
                if sync_os_clipboard {
                    self.exec_os_clipboard(ClipboardOperation::Cut)?;
                } else {
                    self.text_area.cut();
                }
                Ok(true)
            }
            KeyCode::Char('y') => {
                if sync_os_clipboard {
                    self.exec_os_clipboard(ClipboardOperation::Copy)?;
                } else {
                    self.text_area.copy();
                }
                self.set_editor_mode(EditorMode::Normal);
                Ok(true)
            }
            KeyCode::Char('c') => {
                if sync_os_clipboard {
                    self.exec_os_clipboard(ClipboardOperation::Copy)?;
                } else {
                    self.text_area.cut();
                }
                self.set_editor_mode(EditorMode::Insert);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_vim_motions(&mut self, input: &Input, sync_os_clipboard: bool) -> anyhow::Result<()> {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);

        match (input.key_code, has_control) {
            (KeyCode::Char('h'), false) => {
                self.text_area.move_cursor(CursorMove::Back);
            }
            (KeyCode::Char('j'), false) => {
                self.text_area.move_cursor(CursorMove::Down);
            }
            (KeyCode::Char('k'), false) => {
                self.text_area.move_cursor(CursorMove::Up);
            }
            (KeyCode::Char('l'), false) => {
                self.text_area.move_cursor(CursorMove::Forward);
            }
            (KeyCode::Char('w'), false) | (KeyCode::Char('e'), false) => {
                self.text_area.move_cursor(CursorMove::WordForward);
            }
            (KeyCode::Char('b'), false) => {
                self.text_area.move_cursor(CursorMove::WordBack);
            }
            (KeyCode::Char('^'), false) => {
                self.text_area.move_cursor(CursorMove::Head);
            }
            (KeyCode::Char('$'), false) => {
                self.text_area.move_cursor(CursorMove::End);
            }
            (KeyCode::Char('D'), false) => {
                self.text_area.delete_line_by_end();
                self.exec_os_clipboard(ClipboardOperation::Copy)?;
            }
            (KeyCode::Char('C'), false) => {
                self.text_area.delete_line_by_end();
                self.exec_os_clipboard(ClipboardOperation::Copy)?;
                self.mode = EditorMode::Insert;
            }
            (KeyCode::Char('p'), false) => {
                if sync_os_clipboard {
                    self.exec_os_clipboard(ClipboardOperation::Paste)?;
                } else {
                    self.text_area.paste();
                }
            }
            (KeyCode::Char('u'), false) => {
                self.text_area.undo();
            }
            (KeyCode::Char('r'), true) => {
                self.text_area.redo();
            }
            (KeyCode::Char('x'), false) => {
                self.text_area.delete_next_char();
                self.exec_os_clipboard(ClipboardOperation::Copy)?;
            }
            (KeyCode::Char('i'), false) => self.mode = EditorMode::Insert,
            (KeyCode::Char('a'), false) => {
                self.text_area.move_cursor(CursorMove::Forward);
                self.mode = EditorMode::Insert;
            }
            (KeyCode::Char('A'), false) => {
                self.text_area.move_cursor(CursorMove::End);
                self.mode = EditorMode::Insert;
            }
            (KeyCode::Char('o'), false) => {
                self.text_area.move_cursor(CursorMove::End);
                self.text_area.insert_newline();
                self.mode = EditorMode::Insert;
            }
            (KeyCode::Char('O'), false) => {
                self.text_area.move_cursor(CursorMove::Head);
                self.text_area.insert_newline();
                self.text_area.move_cursor(CursorMove::Up);
                self.mode = EditorMode::Insert;
            }
            (KeyCode::Char('I'), false) => {
                self.text_area.move_cursor(CursorMove::Head);
                self.mode = EditorMode::Insert;
            }
            (KeyCode::Char('d'), true) => {
                self.text_area.scroll(Scrolling::HalfPageDown);
            }
            (KeyCode::Char('u'), true) => {
                self.text_area.scroll(Scrolling::HalfPageUp);
            }
            (KeyCode::Char('f'), true) => {
                self.text_area.scroll(Scrolling::PageDown);
            }
            (KeyCode::Char('b'), true) => {
                self.text_area.scroll(Scrolling::PageUp);
            }
            _ => {}
        }

        Ok(())
    }

    pub fn get_editor_mode(&self) -> EditorMode {
        self.mode
    }

    pub fn set_editor_mode(&mut self, mode: EditorMode) {
        match (self.mode, mode) {
            (EditorMode::Normal, EditorMode::Visual) => {
                self.text_area.start_selection();
            }
            (EditorMode::Visual, EditorMode::Normal | EditorMode::Insert) => {
                self.text_area.cancel_selection();
            }
            _ => {}
        }

        self.mode = mode;
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let mut title = "Content".to_owned();
        if self.is_active {
            let mode_caption = match self.mode {
                EditorMode::Normal => " - NORMAL",
                EditorMode::Insert => " - EDIT",
                EditorMode::Visual => " - Visual",
            };
            title.push_str(mode_caption);
        }
        if self.has_unsaved {
            title.push_str(" *");
        }

        let text_block_style = match (self.mode, self.is_active) {
            (EditorMode::Insert, _) => Style::default()
                .fg(EDITOR_MODE_COLOR)
                .add_modifier(Modifier::BOLD),
            (EditorMode::Visual, _) => Style::default()
                .fg(VISUAL_MODE_COLOR)
                .add_modifier(Modifier::BOLD),
            (EditorMode::Normal, true) => Style::default()
                .fg(ACTIVE_CONTROL_COLOR)
                .add_modifier(Modifier::BOLD),
            (EditorMode::Normal, false) => Style::default().fg(INACTIVE_CONTROL_COLOR),
        };

        self.text_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(text_block_style)
                .title(title),
        );

        let mut cursor_style = Style::default();
        if self.is_active {
            cursor_style = match self.mode {
                EditorMode::Normal => cursor_style.bg(Color::White).fg(Color::Black),
                EditorMode::Insert => cursor_style.bg(EDITOR_MODE_COLOR).fg(Color::Black),
                EditorMode::Visual => cursor_style.bg(VISUAL_MODE_COLOR).fg(Color::Black),
            };
        }
        self.text_area.set_cursor_style(cursor_style);

        self.text_area.set_cursor_line_style(Style::default());

        self.text_area.set_style(
            Style::default()
                .fg(Color::Reset)
                .remove_modifier(Modifier::BOLD),
        );

        self.text_area
            .set_selection_style(Style::default().bg(Color::White).fg(Color::Black));

        frame.render_widget(&self.text_area, area);

        self.render_vertical_scrollbar(frame, area);
        self.render_horizontal_scrollbar(frame, area);
    }

    pub fn render_vertical_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        let lines_count = self.text_area.lines().len();

        if lines_count as u16 <= area.height - 2 {
            return;
        }

        let (row, _) = self.text_area.cursor();

        let mut state = ScrollbarState::default()
            .content_length(lines_count)
            .position(row);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some(symbols::line::VERTICAL))
            .thumb_symbol(symbols::block::FULL);

        let scroll_area = area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        });

        frame.render_stateful_widget(scrollbar, scroll_area, &mut state);
    }

    pub fn render_horizontal_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        let max_width = self
            .text_area
            .lines()
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or_default();

        if max_width as u16 <= area.width - 2 {
            return;
        }

        let (_, col) = self.text_area.cursor();

        let mut state = ScrollbarState::default()
            .content_length(max_width)
            .position(col);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(Some("◄"))
            .end_symbol(Some("►"))
            .track_symbol(Some(symbols::line::HORIZONTAL))
            .thumb_symbol("🬋");

        let scroll_area = area.inner(Margin {
            horizontal: 1,
            vertical: 0,
        });

        frame.render_stateful_widget(scrollbar, scroll_area, &mut state);
    }

    pub fn set_active(&mut self, active: bool) {
        if !active && self.is_visual_mode() {
            self.set_editor_mode(EditorMode::Normal);
        }

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

    pub fn set_entry_content<D: DataProvider>(&mut self, entry_content: &str, app: &App<D>) {
        self.is_dirty = true;
        let lines = entry_content.lines().map(|line| line.to_owned()).collect();
        let mut text_area = TextArea::new(lines);
        text_area.move_cursor(tui_textarea::CursorMove::Bottom);
        text_area.move_cursor(tui_textarea::CursorMove::End);

        self.text_area = text_area;

        self.refresh_has_unsaved(app);
    }

    pub fn exec_os_clipboard(
        &mut self,
        operation: ClipboardOperation,
    ) -> anyhow::Result<HandleInputReturnType> {
        let mut clipboard = Clipboard::new().map_err(map_clipboard_error)?;

        match operation {
            ClipboardOperation::Copy => {
                self.text_area.copy();
                let selected_text = self.text_area.yank_text();
                clipboard
                    .set_text(selected_text)
                    .map_err(map_clipboard_error)?;
            }
            ClipboardOperation::Cut => {
                if self.text_area.cut() {
                    self.is_dirty = true;
                    self.has_unsaved = true;
                }
                let selected_text = self.text_area.yank_text();
                clipboard
                    .set_text(selected_text)
                    .map_err(map_clipboard_error)?;
            }
            ClipboardOperation::Paste => {
                let content = clipboard.get_text().map_err(map_clipboard_error)?;
                if content.is_empty() {
                    return Ok(HandleInputReturnType::Handled);
                }

                if !self.text_area.insert_str(content) {
                    bail!("Text can't be pasted into editor")
                }
                self.is_dirty = true;
                self.has_unsaved = true;
            }
        }

        Ok(HandleInputReturnType::Handled)
    }
}

fn is_default_navigation(input: &Input) -> bool {
    let has_control = input.modifiers.contains(KeyModifiers::CONTROL);
    let has_alt = input.modifiers.contains(KeyModifiers::ALT);
    match input.key_code {
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
        KeyCode::Char('f') if !has_control && has_alt => true,
        KeyCode::Char('b') if !has_control && has_alt => true,
        KeyCode::Char('e') if has_control || has_alt => true,
        KeyCode::Char('a') if has_control || has_alt => true,
        KeyCode::Char('v') if has_control || has_alt => true,
        _ => false,
    }
}

fn map_clipboard_error(err: arboard::Error) -> anyhow::Error {
    anyhow!(
        "Error while communicating with the operation system clipboard.\nError Details: {}",
        err.to_string()
    )
}
