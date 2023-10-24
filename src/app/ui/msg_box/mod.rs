use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::keymap::Input;

use super::ui_functions::centered_rect_exact_height;

// Not all enums are used in this app at this point
#[allow(dead_code)]
#[derive(Debug)]
pub enum MsgBoxType {
    Error(String),
    Warning(String),
    Info(String),
    Question(String),
}

// Not all enums are used in this app at this point
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgBoxActions {
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgBoxResult {
    Ok,
    Cancel,
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgBoxInputResult {
    Keep,
    Close(MsgBoxResult),
}

#[derive(Debug)]
pub struct MsgBox {
    msg_type: MsgBoxType,
    actions: MsgBoxActions,
}

impl MsgBox {
    pub fn new(msg_type: MsgBoxType, actions: MsgBoxActions) -> Self {
        Self { msg_type, actions }
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let area = centered_rect_exact_height(55, 8, area);

        let (title, color, text) = match &self.msg_type {
            MsgBoxType::Error(text) => ("Error", Color::LightRed, text),
            MsgBoxType::Warning(text) => ("Warning", Color::Yellow, text),
            MsgBoxType::Info(text) => ("Info", Color::LightGreen, text),
            MsgBoxType::Question(text) => ("", Color::LightBlue, text),
        };

        let border = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(color));

        frame.render_widget(Clear, area);
        frame.render_widget(border, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(3)
            .vertical_margin(2)
            .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
            .split(area);

        let text_paragraph = Paragraph::new(Span::raw(text))
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(text_paragraph, chunks[0]);

        let actions_text = match self.actions {
            MsgBoxActions::Ok => "(O)k",
            MsgBoxActions::OkCancel => "(O)k , (C)ancel",
            MsgBoxActions::YesNo => "(Y)es , (N)o",
            MsgBoxActions::YesNoCancel => "(Y)es , (N)o , (C)ancel",
        };

        let actions_paragraph = Paragraph::new(Span::raw(actions_text))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(actions_paragraph, chunks[1]);
    }

    pub fn handle_input(&self, input: &Input) -> MsgBoxInputResult {
        match self.actions {
            MsgBoxActions::Ok => match input.key_code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('o') => {
                    MsgBoxInputResult::Close(MsgBoxResult::Ok)
                }
                _ => MsgBoxInputResult::Keep,
            },
            MsgBoxActions::OkCancel => match input.key_code {
                KeyCode::Enter | KeyCode::Char('o') => MsgBoxInputResult::Close(MsgBoxResult::Ok),
                KeyCode::Esc | KeyCode::Char('c') => MsgBoxInputResult::Close(MsgBoxResult::Cancel),
                _ => MsgBoxInputResult::Keep,
            },
            MsgBoxActions::YesNo => match input.key_code {
                KeyCode::Enter | KeyCode::Char('y') => MsgBoxInputResult::Close(MsgBoxResult::Yes),
                KeyCode::Esc | KeyCode::Char('n') => MsgBoxInputResult::Close(MsgBoxResult::No),
                _ => MsgBoxInputResult::Keep,
            },
            MsgBoxActions::YesNoCancel => match input.key_code {
                KeyCode::Enter | KeyCode::Char('y') => MsgBoxInputResult::Close(MsgBoxResult::Yes),
                KeyCode::Char('n') => MsgBoxInputResult::Close(MsgBoxResult::No),
                KeyCode::Esc | KeyCode::Char('c') => MsgBoxInputResult::Close(MsgBoxResult::Cancel),
                _ => MsgBoxInputResult::Keep,
            },
        }
    }
}
