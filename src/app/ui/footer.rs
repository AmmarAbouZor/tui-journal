use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{commands::UICommand, keymap::Keymap};

pub fn render_footer<B: Backend>(frame: &mut Frame<B>, area: Rect, global_keymaps: &[Keymap]) {
    let close_keymap = global_keymaps
        .iter()
        .find(|keymap| keymap.command == UICommand::Quit)
        .expect("Quit command must be in glabal commands");

    let help_keymap = global_keymaps
        .iter()
        .find(|keymap| keymap.command == UICommand::ShowHelp)
        .expect("ShowHelp command must be in glabal commands");

    let spans = Spans::from(vec![
        get_keymap_spans(&close_keymap),
        Span::raw(" | "),
        get_keymap_spans(&help_keymap),
    ]);

    let footer = Paragraph::new(spans).alignment(Alignment::Left).block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default()),
    );

    frame.render_widget(footer, area);
}

fn get_keymap_spans(keymap: &Keymap) -> Span {
    Span::styled(
        format!("{}: {}", keymap.command.get_info().name, keymap.key),
        Style::default(),
    )
}
