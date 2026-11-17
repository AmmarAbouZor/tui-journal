use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Rect, Size},
    text::Line,
    widgets::{Paragraph, Wrap},
};

use crate::app::keymap::Input;

mod markdown;
mod table;

pub struct Preview {
    source: String,
    rows: Vec<Row>,
    viewport: Size,
    offset: usize,
    max_offset: usize,
}

struct Row {
    columns: Vec<(u16, Paragraph<'static>)>,
    height: usize,
}

impl Preview {
    pub fn new(source: String) -> Self {
        Self {
            source,
            rows: Vec::new(),
            viewport: Size::default(),
            offset: 0,
            max_offset: 0,
        }
    }

    pub fn handle_input(&mut self, input: &Input) {
        if !input.modifiers.is_empty() {
            return;
        }
        let page = usize::from(self.viewport.height);
        self.offset = match input.key_code {
            KeyCode::Up | KeyCode::Char('k') => self.offset.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.offset.saturating_add(1),
            KeyCode::PageUp => self.offset.saturating_sub(page),
            KeyCode::PageDown => self.offset.saturating_add(page),
            KeyCode::Home => 0,
            KeyCode::End => self.max_offset,
            _ => return,
        }
        .min(self.max_offset);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if area.is_empty() {
            return;
        }
        if self.viewport != area.as_size() {
            if self.viewport.width != area.width {
                self.rows = markdown::layout(&self.source, area.width);
            }
            self.viewport = area.as_size();
            let height: usize = self.rows.iter().map(|row| row.height).sum();
            self.max_offset = height.saturating_sub(usize::from(area.height));
            self.offset = self.offset.min(self.max_offset);
        }
        let mut skip = self.offset;
        let mut y = area.y;
        for row in &mut self.rows {
            if skip >= row.height {
                skip -= row.height;
                continue;
            }
            let height = (row.height - skip).min(usize::from(area.bottom() - y)) as u16;
            row.render(frame, Rect::new(area.x, y, area.width, height), skip);
            skip = 0;
            y += height;
            if y == area.bottom() {
                break;
            }
        }
    }
}

impl Row {
    fn new(columns: Vec<(u16, Paragraph<'static>)>) -> Self {
        let height = columns
            .iter()
            .map(|(width, cell)| cell.line_count(*width))
            .max()
            .unwrap_or(0)
            .max(1);
        Self { columns, height }
    }

    fn text(line: Line<'static>, width: u16) -> Self {
        Self::new(vec![(
            width,
            Paragraph::new(line).wrap(Wrap { trim: false }),
        )])
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, skip: usize) {
        let mut x = area.x;
        for (i, (width, cell)) in self.columns.iter_mut().enumerate() {
            if i != 0 {
                for y in area.y..area.bottom() {
                    frame.buffer_mut()[(x + 1, y)].set_char('│');
                }
                x += table::GUTTER;
            }
            let cell_area = Rect::new(x, area.y, *width, area.height);
            *cell = std::mem::take(cell).scroll((skip.try_into().unwrap_or(u16::MAX), 0));
            frame.render_widget(&*cell, cell_area);
            x += *width;
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyModifiers;
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, style::Modifier};

    use super::*;

    fn draw(preview: &mut Preview, width: u16, height: u16) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|f| preview.render(f, f.area()))
            .unwrap()
            .buffer
            .clone()
    }

    fn row(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }

    #[test]
    fn control_characters_render_as_spaces() {
        let mut preview = Preview::new(
            "A&#7;B `C\u{1b}D` [link](a&#7;b)\n\n| Data |\n| --- |\n| E\u{85}F |".into(),
        );
        let buffer = draw(&mut preview, 40, 4);
        assert_eq!(row(&buffer, 0), "A B C D link (a b)");
        assert_eq!(row(&buffer, 3), "E F");
    }

    #[test]
    fn nested_blocks_preserve_list_lines() {
        for block in ["> after", "# after", "```\n  after\n  ```"] {
            let mut preview = Preview::new(format!("- before\n  {block}"));
            let buffer = draw(&mut preview, 20, 2);
            assert_eq!(row(&buffer, 0), "• before");
            assert!(row(&buffer, 1).ends_with("after"));
        }

        let mut preview = Preview::new("- ```\n  first\n  ```".into());
        assert_eq!(row(&draw(&mut preview, 20, 1), 0), "• first");
    }

    #[test]
    fn single_newline_breaks_the_line_not_the_paragraph() {
        let mut preview = Preview::new("first paragraph\n\nnow what what\nthe what".into());
        let buffer = draw(&mut preview, 32, 4);
        assert_eq!(row(&buffer, 1), "");
        assert_eq!(row(&buffer, 2), "now what what");
        assert_eq!(row(&buffer, 3), "the what");
    }

    #[test]
    fn table_keeps_inline_styles_and_following_blocks_separate() {
        let mut preview = Preview::new(
            "| 品名 | Qty |\n| :---: | ---: |\n| ***茶*** | 2 |\n\n- [x] packed `tea`".into(),
        );
        let buffer = draw(&mut preview, 24, 5);
        assert_eq!(buffer[(1, 1)].symbol(), "茶");
        assert!(
            buffer[(1, 1)]
                .modifier
                .contains(Modifier::BOLD | Modifier::ITALIC)
        );
        assert_eq!(buffer[(5, 1)].symbol(), "│");
        assert_eq!(row(&buffer, 3), "• [x] packed tea");
        assert!(
            !buffer[(0, 3)]
                .modifier
                .intersects(Modifier::BOLD | Modifier::ITALIC)
        );
    }

    #[test]
    fn table_scrolls_through_wrapped_cells_and_reflows_on_resize() {
        let mut preview = Preview::new("| Day | Count | Note |\n| --- | ---: | --- |\n| Mo | 3 | **green** moss grew |\n| Tu | 12 | dry |".into());
        let wide = draw(&mut preview, 24, 2);
        assert_eq!(row(&wide, 0), "Day │ Count │ Note");
        assert_eq!(row(&wide, 1), "Mo  │     3 │ green moss");
        assert!(wide[(14, 1)].modifier.contains(Modifier::BOLD));

        preview.handle_input(&Input::new(KeyCode::End, KeyModifiers::NONE));
        let bottom = draw(&mut preview, 24, 2);
        assert_eq!(row(&bottom, 0), "    │       │ grew");
        assert_eq!(row(&bottom, 1), "Tu  │    12 │ dry");

        preview.handle_input(&Input::new(KeyCode::Home, KeyModifiers::NONE));
        let narrow = draw(&mut preview, 14, 12);
        assert_eq!(row(&narrow, 0), "Day: Mo");
        assert_eq!(row(&narrow, 1), "Count: 3");
        assert_eq!(row(&narrow, 2), "Note: green");
        assert_eq!(row(&narrow, 3), "moss grew");
        assert_eq!(row(&narrow, 7), "Note: dry");
    }
}
