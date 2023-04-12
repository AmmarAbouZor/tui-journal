use chrono::Datelike;

use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState};
use tui::Frame;

use crate::data::Entry;

use super::ACTIVE_CONTROL_COLOR;
use super::INACTIVE_CONTROL_COLOR;

mod command_actions;

pub(crate) use command_actions::execute_command;

pub struct EntriesList {
    pub state: ListState,
    is_active: bool,
}

impl<'a> EntriesList {
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            is_active: false,
        }
    }

    pub fn get_widget(&self, entries: &'a [Entry]) -> List<'a> {
        let items: Vec<ListItem> = entries
            .iter()
            .map(|entry| {
                let spans = vec![
                    Spans::from(Span::styled(
                        entry.title.as_str(),
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Spans::from(Span::styled(
                        format!(
                            "{},{},{}",
                            entry.date.day(),
                            entry.date.month(),
                            entry.date.year()
                        ),
                        Style::default().add_modifier(Modifier::DIM),
                    )),
                ];

                ListItem::new(spans)
            })
            .collect();

        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Journals")
                    .border_style(match self.is_active {
                        true => Style::default().fg(ACTIVE_CONTROL_COLOR),
                        false => Style::default().fg(INACTIVE_CONTROL_COLOR),
                    }),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
    }

    pub fn render_widget<B: Backend>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        entries: &'a [Entry],
    ) {
        let entries_widget = self.get_widget(entries);

        frame.render_stateful_widget(entries_widget, area, &mut self.state);
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}
