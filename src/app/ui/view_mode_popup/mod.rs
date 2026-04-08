use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::keymap::Input;

use super::{PopupReturn, ui_functions::centered_rect};

/// Which view mode the user chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Flat,
    Folder,
}

/// Small popup that lets the user pick between Flat and Folder view.
#[derive(Debug)]
pub struct ViewModePopup {
    current: ViewMode,
    hovered: ViewMode,
}

impl ViewModePopup {
    pub fn new(current: ViewMode) -> Self {
        Self {
            current,
            hovered: current,
        }
    }

    pub fn render_widget(&self, frame: &mut Frame, area: Rect, styles: &super::Styles) {
        let area = centered_rect(60, 40, area);
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Switch View Mode ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(styles.journals_list.block_active);

        frame.render_widget(block, area);

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(3)
            .vertical_margin(2)
            .constraints([
                Constraint::Length(1), // hint
                Constraint::Length(1), // spacer
                Constraint::Length(3), // option 1
                Constraint::Length(1), // spacer
                Constraint::Length(3), // option 2
                Constraint::Length(1), // spacer
                Constraint::Min(1),    // footer
            ])
            .split(area);

        // Hint line
        let hint = Paragraph::new("Press 1 or 2 to switch  ·  Esc/q to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(hint, inner[0]);

        self.render_option(frame, inner[2], ViewMode::Flat, "[1] Flat View", "Show all journals in one flat list (current default)");
        self.render_option(frame, inner[4], ViewMode::Folder, "[2] Folder View", "Browse journals through tag-based folders  (e.g. linux.ubuntu → linux/ubuntu)");

        // Current mode indicator
        let current_label = match self.current {
            ViewMode::Flat => "Currently: Flat View",
            ViewMode::Folder => "Currently: Folder View",
        };
        let footer = Paragraph::new(current_label)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC));
        frame.render_widget(footer, inner[6]);
    }

    fn render_option(
        &self,
        frame: &mut Frame,
        area: Rect,
        mode: ViewMode,
        label: &str,
        description: &str,
    ) {
        let is_hovered = self.hovered == mode;
        let is_current = self.current == mode;

        let border_style = if is_hovered {
            Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let label_style = if is_current {
            Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)
        } else if is_hovered {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let active_marker = if is_current { " ✓" } else { "" };
        let title = format!("{label}{active_marker}");

        let block = Block::default()
            .title(Span::styled(title, label_style))
            .borders(Borders::ALL)
            .border_style(border_style);

        let desc = Paragraph::new(Line::from(Span::styled(
            description,
            Style::default().fg(Color::Gray),
        )))
        .block(block)
        .wrap(Wrap { trim: true });

        frame.render_widget(desc, area);
    }

    pub fn handle_input(&mut self, input: &Input) -> PopupReturn<ViewMode> {
        let has_ctrl = input.modifiers.contains(KeyModifiers::CONTROL);
        match input.key_code {
            KeyCode::Esc | KeyCode::Char('q') => PopupReturn::Cancel,
            KeyCode::Char('c') if has_ctrl => PopupReturn::Cancel,
            KeyCode::Char('1') => PopupReturn::Apply(ViewMode::Flat),
            KeyCode::Char('2') => PopupReturn::Apply(ViewMode::Folder),
            KeyCode::Down | KeyCode::Char('j') => {
                self.hovered = ViewMode::Folder;
                PopupReturn::KeepPopup
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.hovered = ViewMode::Flat;
                PopupReturn::KeepPopup
            }
            KeyCode::Enter => PopupReturn::Apply(self.hovered),
            _ => PopupReturn::KeepPopup,
        }
    }
}
