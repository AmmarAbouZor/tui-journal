use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_rect_exact_height(percent_x: u16, height: u16, r: Rect) -> Rect {
    let height_percentage = ((height as f32 / r.height as f32) * 100f32) as u16;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height_percentage) / 2),
                Constraint::Length(height),
                Constraint::Percentage((100 - height_percentage) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_rect_exact_dimensions(width: u16, height: u16, r: Rect) -> Rect {
    let height_percentage = ((height as f32 / r.height as f32) * 100f32) as u16;
    let width_percentage = ((width as f32 / r.width as f32) * 100f32) as u16;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height_percentage) / 2),
                Constraint::Length(height),
                Constraint::Percentage((100 - height_percentage) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - width_percentage) / 2),
                Constraint::Percentage(width),
                Constraint::Percentage((100 - width_percentage) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn render_message_centered<B: Backend>(frame: &mut Frame<B>, message: &str) {
    const TEXT_MARGINE: u16 = 4;
    const HEIGHT_MARGINE: u16 = 2;

    let width = message.len() as u16 + TEXT_MARGINE;
    let height = message.lines().count() as u16 + HEIGHT_MARGINE;

    let area = centered_rect_exact_dimensions(width, height, frame.size());

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}
