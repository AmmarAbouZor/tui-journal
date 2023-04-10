use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Clear},
    Frame,
};

use super::{ui_functions::centered_rect, UIComponents};

pub fn render_help_popup<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    ui_components: &UIComponents,
) {
    let test_block = Block::default()
        .title("Test Control help")
        .borders(Borders::ALL);

    let area = centered_rect(80, 80, area);

    frame.render_widget(Clear, area);
    frame.render_widget(test_block, area);
}
