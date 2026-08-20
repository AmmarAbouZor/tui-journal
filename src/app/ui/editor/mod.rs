use anyhow::{anyhow, bail};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    prelude::Margin,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::{App, keymap::Input, runner::HandleInputReturnType};

use backend::DataProvider;
use tui_textarea::{CursorMove, Scrolling, TextArea};

use super::Styles;
use super::commands::ClipboardOperation;

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

/// Direction of a single-line vertical cursor movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerticalDir {
    Up,
    Down,
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
                let handled = match input.key_code {
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

                if handled {
                    return Ok(HandleInputReturnType::Handled);
                }
            }

            if let Some(dir) = vertical_arrow_dir(input) {
                self.move_cursor_vertical(dir, self.should_extend_selection(input));
                return Ok(HandleInputReturnType::Handled);
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

        if let Some(dir) = vertical_arrow_dir(input) {
            self.move_cursor_vertical(dir, self.should_extend_selection(input));
        } else if is_default_navigation(input) {
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
                self.move_cursor_down(self.is_visual_mode());
            }
            (KeyCode::Char('k'), false) => {
                self.move_cursor_up(self.is_visual_mode());
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

    /// Moves the cursor up one line, snapping to the line start when already on the
    /// first line so the key press is never a dead no-op. Extends the active
    /// selection when `select` is set.
    fn move_cursor_up(&mut self, select: bool) {
        self.move_cursor_vertical(VerticalDir::Up, select);
    }

    /// Moves the cursor down one line, snapping to the line end when already on the
    /// last line so the key press is never a dead no-op. Extends the active
    /// selection when `select` is set.
    fn move_cursor_down(&mut self, select: bool) {
        self.move_cursor_vertical(VerticalDir::Down, select);
    }

    /// Single entry point for vertical cursor movement across all modes. Owns the
    /// selection state, the snap-to-edge decision, and the move itself so callers
    /// never fall back to the underlying text area's default navigation.
    fn move_cursor_vertical(&mut self, dir: VerticalDir, select: bool) {
        let Some(cursor_move) = self.resolve_vertical_move(dir) else {
            return;
        };

        if select {
            if !self.text_area.is_selecting() {
                self.text_area.start_selection();
            }
        } else {
            self.text_area.cancel_selection();
        }

        self.text_area.move_cursor(cursor_move);
    }

    /// Resolves a direction to the concrete move to apply, snapping to the line
    /// edge on the first and last lines. Yields `None` when the cursor already
    /// sits on the edge it would snap to, so a dead key press leaves both the
    /// cursor and the selection untouched.
    fn resolve_vertical_move(&self, dir: VerticalDir) -> Option<CursorMove> {
        let (row, col) = self.text_area.cursor();
        match dir {
            VerticalDir::Up if self.is_on_first_line() => (col > 0).then_some(CursorMove::Head),
            VerticalDir::Up => Some(CursorMove::Up),
            VerticalDir::Down if self.is_on_last_line() => {
                let line_end = self.text_area.lines()[row].chars().count();
                (col < line_end).then_some(CursorMove::End)
            }
            VerticalDir::Down => Some(CursorMove::Down),
        }
    }

    /// Whether a vertical move should extend a selection: always in visual mode,
    /// and in any other mode while Shift is held, as a conventional editor does.
    fn should_extend_selection(&self, input: &Input) -> bool {
        self.is_visual_mode() || input.modifiers.contains(KeyModifiers::SHIFT)
    }

    fn is_on_first_line(&self) -> bool {
        self.text_area.cursor().0 == 0
    }

    fn is_on_last_line(&self) -> bool {
        let (row, _) = self.text_area.cursor();
        row + 1 >= self.text_area.lines().len()
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

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
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

        let estyles = &styles.editor;

        let text_block_style = match (self.mode, self.is_active) {
            (EditorMode::Insert, _) => estyles.block_insert,
            (EditorMode::Visual, _) => estyles.block_visual,
            (EditorMode::Normal, true) => estyles.block_normal_active,
            (EditorMode::Normal, false) => estyles.block_normal_inactive,
        };

        self.text_area.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(text_block_style)
                .title(title),
        );

        let cursor_style = if self.is_active {
            let s = match self.mode {
                EditorMode::Normal => estyles.cursor_normal,
                EditorMode::Insert => estyles.cursor_insert,
                EditorMode::Visual => estyles.cursor_visual,
            };
            Style::from(s)
        } else {
            Style::reset()
        };
        self.text_area.set_cursor_style(cursor_style);

        self.text_area.set_cursor_line_style(Style::reset());

        self.text_area.set_style(Style::reset());

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

/// Maps a plain or Shift-modified Up/Down arrow to a vertical direction so that
/// vertical movement is fully owned by [`Editor::move_cursor_vertical`]. Arrows
/// carrying Ctrl/Alt keep their default-navigation handling.
fn vertical_arrow_dir(input: &Input) -> Option<VerticalDir> {
    if !(input.modifiers - KeyModifiers::SHIFT).is_empty() {
        return None;
    }
    match input.key_code {
        KeyCode::Up => Some(VerticalDir::Up),
        KeyCode::Down => Some(VerticalDir::Down),
        _ => None,
    }
}

fn is_default_navigation(input: &Input) -> bool {
    let has_control = input.modifiers.contains(KeyModifiers::CONTROL);
    let has_alt = input.modifiers.contains(KeyModifiers::ALT);
    match input.key_code {
        KeyCode::Left
        | KeyCode::Right
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
    anyhow!("Error while communicating with the operation system clipboard.\nError Details: {err}",)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::keymap::Input;
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui_textarea::{CursorMove, TextArea};

    fn editor_with(lines: &[&str], row: u16, col: u16) -> Editor<'static> {
        let mut editor = Editor::new();
        editor.text_area = TextArea::new(lines.iter().map(|l| (*l).to_owned()).collect());
        editor.text_area.move_cursor(CursorMove::Jump(row, col));
        editor
    }

    #[test]
    fn vertical_arrow_dir_maps_plain_and_shift_arrows() {
        assert_eq!(
            vertical_arrow_dir(&Input::new(KeyCode::Up, KeyModifiers::NONE)),
            Some(VerticalDir::Up)
        );
        assert_eq!(
            vertical_arrow_dir(&Input::new(KeyCode::Down, KeyModifiers::NONE)),
            Some(VerticalDir::Down)
        );
        assert_eq!(
            vertical_arrow_dir(&Input::new(KeyCode::Up, KeyModifiers::SHIFT)),
            Some(VerticalDir::Up)
        );
    }

    #[test]
    fn vertical_arrow_dir_ignores_ctrl_alt_and_other_keys() {
        assert_eq!(
            vertical_arrow_dir(&Input::new(KeyCode::Up, KeyModifiers::CONTROL)),
            None
        );
        assert_eq!(
            vertical_arrow_dir(&Input::new(KeyCode::Down, KeyModifiers::ALT)),
            None
        );
        assert_eq!(
            vertical_arrow_dir(&Input::new(KeyCode::Left, KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            vertical_arrow_dir(&Input::new(KeyCode::Char('k'), KeyModifiers::NONE)),
            None
        );
    }

    #[test]
    fn up_on_first_line_snaps_to_head() {
        let mut editor = editor_with(&["hello", "world"], 0, 3);
        editor.move_cursor_up(false);
        assert_eq!(editor.text_area.cursor(), (0, 0));
        assert!(!editor.text_area.is_selecting());
    }

    #[test]
    fn up_mid_buffer_moves_one_line_without_snapping() {
        let mut editor = editor_with(&["a", "bb", "ccc"], 2, 1);
        editor.move_cursor_up(false);
        assert_eq!(editor.text_area.cursor().0, 1);
    }

    #[test]
    fn down_on_last_line_snaps_to_end() {
        let mut editor = editor_with(&["hello", "world"], 1, 0);
        editor.move_cursor_down(false);
        assert_eq!(editor.text_area.cursor(), (1, 5));
    }

    #[test]
    fn down_mid_buffer_moves_one_line_without_snapping() {
        let mut editor = editor_with(&["a", "bb", "ccc"], 0, 0);
        editor.move_cursor_down(false);
        assert_eq!(editor.text_area.cursor().0, 1);
    }

    #[test]
    fn snap_with_select_extends_selection() {
        let mut editor = editor_with(&["hello", "world"], 0, 3);
        editor.move_cursor_up(true);
        assert_eq!(editor.text_area.cursor(), (0, 0));
        assert!(editor.text_area.is_selecting());
    }

    #[test]
    fn move_without_select_cancels_existing_selection() {
        let mut editor = editor_with(&["a", "bb", "ccc"], 2, 1);
        editor.text_area.start_selection();
        editor.move_cursor_up(false);
        assert!(!editor.text_area.is_selecting());
    }

    #[test]
    fn shift_selects_in_normal_mode_without_leaving_it() {
        let mut editor = editor_with(&["hello", "world"], 0, 3);
        editor.mode = EditorMode::Normal;

        editor.move_cursor_vertical(
            VerticalDir::Up,
            editor.should_extend_selection(&Input::new(KeyCode::Up, KeyModifiers::SHIFT)),
        );

        assert_eq!(editor.text_area.cursor(), (0, 0));
        assert_eq!(editor.text_area.selection_range(), Some(((0, 0), (0, 3))));
        assert_eq!(editor.mode, EditorMode::Normal);
    }

    #[test]
    fn dead_key_press_leaves_selection_untouched() {
        let mut editor = editor_with(&["hello", "world"], 0, 0);
        editor.move_cursor_up(true);
        assert!(!editor.text_area.is_selecting());

        let mut editor = editor_with(&["hello", "world"], 1, 5);
        editor.move_cursor_down(true);
        assert!(!editor.text_area.is_selecting());
    }

    #[test]
    fn dead_key_press_preserves_an_existing_selection() {
        let mut editor = editor_with(&["hello", "world"], 0, 3);
        editor.move_cursor_up(true);
        let selection = editor.text_area.selection_range();

        editor.move_cursor_up(true);

        assert_eq!(editor.text_area.selection_range(), selection);
    }

    #[test]
    fn should_extend_selection_depends_on_mode_and_shift() {
        let mut editor = editor_with(&["abc"], 0, 0);

        editor.mode = EditorMode::Normal;
        assert!(editor.should_extend_selection(&Input::new(KeyCode::Up, KeyModifiers::SHIFT)));
        assert!(!editor.should_extend_selection(&Input::new(KeyCode::Up, KeyModifiers::NONE)));

        editor.mode = EditorMode::Insert;
        assert!(editor.should_extend_selection(&Input::new(KeyCode::Up, KeyModifiers::SHIFT)));
        assert!(!editor.should_extend_selection(&Input::new(KeyCode::Up, KeyModifiers::NONE)));

        editor.mode = EditorMode::Visual;
        assert!(editor.should_extend_selection(&Input::new(KeyCode::Up, KeyModifiers::NONE)));
    }
}
