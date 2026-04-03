use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{
    keymap::Input,
    ui::{Styles, ui_functions::centered_rect},
};

const FOOTER_TEXT: &str =
    r"Enter or <Ctrl-m>: Confirm | Esc, q or <Ctrl-c>: Cancel";
const FOOTER_MARGINE: u16 = 4;

pub enum FoldersPopupReturn {
    Keep,
    Cancel,
    Apply(String),
}

pub struct FoldersPopup {
    state: ListState,
    folders: Vec<String>,
}

impl FoldersPopup {
    pub fn new(current_folder: &str, mut folders: Vec<String>) -> Self {
        let mut state = ListState::default();

        if !current_folder.is_empty() && !folders.contains(&current_folder.to_string()) {
            folders.insert(0, current_folder.to_string());
        }

        if let Some(idx) = folders.iter().position(|f| f == current_folder) {
            state.select(Some(idx));
        } else if !folders.is_empty() {
            state.select(Some(0));
        }

        Self { state, folders }
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let mut area = centered_rect(70, 100, area);
        area.y += 1;
        area.height -= 2;

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Folders")
            .border_type(BorderType::Rounded);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let footer_height = if area.width < FOOTER_TEXT.len() as u16 + FOOTER_MARGINE {
            3
        } else {
            2
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .vertical_margin(1)
            .constraints([Constraint::Min(3), Constraint::Length(footer_height)].as_ref())
            .split(area);

        if self.folders.is_empty() {
            self.render_place_holder(frame, chunks[0]);
        } else {
            self.render_list(frame, chunks[0], styles);
        }

        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .style(Style::default()),
            );

        frame.render_widget(footer, chunks[1]);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let gstyles = &styles.general;
        let items: Vec<ListItem> = self
            .folders
            .iter()
            .map(|folder| ListItem::new(folder.as_str()).style(Style::reset()))
            .collect();

        let list = List::new(items)
            .highlight_style(gstyles.list_highlight_active)
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_place_holder(&mut self, frame: &mut Frame, area: Rect) {
        let place_holder_text = String::from("\nNo existing folders found");

        let place_holder = Paragraph::new(place_holder_text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(place_holder, area);
    }

    pub fn handle_input(&mut self, input: &Input) -> FoldersPopupReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);
        match input.key_code {
            KeyCode::Char('j') | KeyCode::Down => self.cycle_next(),
            KeyCode::Char('k') | KeyCode::Up => self.cycle_prev(),
            KeyCode::Esc | KeyCode::Char('q') => FoldersPopupReturn::Cancel,
            KeyCode::Char('c') if has_control => FoldersPopupReturn::Cancel,
            KeyCode::Enter => self.confirm(),
            KeyCode::Char('m') if has_control => self.confirm(),
            _ => FoldersPopupReturn::Keep,
        }
    }

    fn cycle_next(&mut self) -> FoldersPopupReturn {
        if !self.folders.is_empty() {
            let last_index = self.folders.len() - 1;
            let new_index = self
                .state
                .selected()
                .map(|idx| if idx >= last_index { 0 } else { idx + 1 })
                .unwrap_or(0);

            self.state.select(Some(new_index));
        }

        FoldersPopupReturn::Keep
    }

    fn cycle_prev(&mut self) -> FoldersPopupReturn {
        if !self.folders.is_empty() {
            let last_index = self.folders.len() - 1;
            let new_index = self
                .state
                .selected()
                .map(|idx| idx.checked_sub(1).unwrap_or(last_index))
                .unwrap_or(last_index);

            self.state.select(Some(new_index));
        }

        FoldersPopupReturn::Keep
    }

    fn confirm(&self) -> FoldersPopupReturn {
        if let Some(idx) = self.state.selected() {
            if let Some(folder) = self.folders.get(idx) {
                return FoldersPopupReturn::Apply(folder.to_owned());
            }
        }

        FoldersPopupReturn::Cancel
    }
}
