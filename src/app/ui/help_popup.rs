use std::collections::BTreeMap;

use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Row, Table},
    Frame,
};

use crate::app::keymap::Input;

use super::{commands::CommandInfo, ui_functions::centered_rect, UICommand, UIComponents};

const KEY_PERC: u16 = 18;
const NAME_PERC: u16 = 27;
const DESCRIPTION_PERC: u16 = 100 - NAME_PERC - KEY_PERC;
const MARGINE: u16 = 8;

pub fn render_help_popup<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    ui_components: &UIComponents,
) {
    let area = centered_rect(90, 80, area);

    let header_cells = ["Key", "Command", "Description"]
        .into_iter()
        .map(|header| Cell::from(header).style(Style::default().fg(Color::LightBlue)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let mut unique_commands: BTreeMap<UICommand, Vec<Input>> = BTreeMap::new();

    ui_components.get_all_keymaps().for_each(|keymap| {
        unique_commands
            .entry(keymap.command)
            .and_modify(|keys| keys.push(keymap.key))
            .or_insert(vec![keymap.key]);
    });

    let rows = unique_commands.into_iter().map(|(command, keys)| {
        let keys: Vec<_> = keys.into_iter().map(|input| input.to_string()).collect();
        let mut keys_text = keys.join(", ");

        let CommandInfo {
            mut name,
            mut description,
        } = command.get_info();

        // Text wrapping
        let keys_width = (area.width - MARGINE) * KEY_PERC / 100;
        let name_width = area.width * NAME_PERC / 100;
        let description_width = (area.width - MARGINE) * DESCRIPTION_PERC / 100;

        keys_text = textwrap::fill(keys_text.as_str(), keys_width as usize);
        name = textwrap::fill(name.as_str(), name_width as usize);
        description = textwrap::fill(description.as_str(), description_width as usize);

        let height = name
            .lines()
            .count()
            .max(description.lines().count())
            .max(keys_text.lines().count()) as u16;

        let cells = vec![
            Cell::from(keys_text).style(Style::default().add_modifier(Modifier::ITALIC)),
            Cell::from(name),
            Cell::from(description),
        ];

        Row::new(cells).height(height)
    });

    let keymaps_table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title("Help - Keybindigs")
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Percentage(KEY_PERC),
            Constraint::Percentage(NAME_PERC),
            Constraint::Percentage(DESCRIPTION_PERC),
        ]);

    frame.render_widget(Clear, area);
    frame.render_widget(keymaps_table, area);
}
