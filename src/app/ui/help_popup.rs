use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table},
    Frame,
};

use super::{commands::CommandInfo, ui_functions::centered_rect, UIComponents};

const KEY_WIDTH: u16 = 10;
const NAME_PERC: u16 = 30;
const DESCRIPTION_PERC: u16 = 70;
const MARGINE: u16 = 8;

pub fn render_help_popup<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    ui_components: &UIComponents,
) {
    let area = centered_rect(80, 80, area);

    let header_cells = ["Key", "Command", "Description"]
        .into_iter()
        .map(|header| Cell::from(header).style(Style::default().fg(Color::LightBlue)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = ui_components.get_all_keymaps().map(|keymap| {
        let key = keymap.key.to_string();
        let CommandInfo {
            mut name,
            mut description,
        } = keymap.command.get_info();

        // Text wrapping
        let name_width = (area.width - KEY_WIDTH) * NAME_PERC / 100;
        let description_width = (area.width - KEY_WIDTH - MARGINE) * DESCRIPTION_PERC / 100;

        name = textwrap::fill(name.as_str(), name_width as usize);
        description = textwrap::fill(description.as_str(), description_width as usize);

        let height = name.lines().count().max(description.lines().count()) as u16;

        let cells = vec![
            Cell::from(key).style(Style::default().add_modifier(Modifier::ITALIC)),
            Cell::from(name),
            Cell::from(description),
        ];

        Row::new(cells).height(height)
    });

    let keymaps_table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title("Test Control help")
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Length(KEY_WIDTH),
            Constraint::Percentage(NAME_PERC),
            Constraint::Percentage(DESCRIPTION_PERC),
        ]);

    frame.render_widget(Clear, area);
    frame.render_widget(keymaps_table, area);
}
