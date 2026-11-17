use pulldown_cmark::{Alignment, Event, TagEnd};
use ratatui::{
    layout,
    style::Stylize,
    text::Line,
    widgets::{Paragraph, Wrap},
};

use super::{Row, markdown::Inlines};

// A vertical rule with one space on either side.
pub const GUTTER: u16 = 3;

pub fn layout<'a>(
    events: &mut impl Iterator<Item = Event<'a>>,
    alignments: &[Alignment],
    width: u16,
) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut cells = Vec::new();
    let mut inline = Inlines::default();
    for event in events {
        match event {
            Event::End(TagEnd::TableCell) => cells.push(std::mem::take(&mut inline.line)),
            Event::End(TagEnd::TableRow | TagEnd::TableHead) => {
                rows.push(std::mem::take(&mut cells))
            }
            Event::End(TagEnd::Table) => break,
            event => inline.event(event),
        }
    }
    let Some(headers) = rows.first() else {
        return Vec::new();
    };
    let Some(widths) = column_widths(headers, &rows, width) else {
        return labeled_rows(rows, width);
    };
    rows.into_iter()
        .enumerate()
        .map(|(i, row)| {
            let columns = row
                .into_iter()
                .zip(&widths)
                .zip(alignments)
                .map(|((cell, width), alignment)| {
                    let alignment = match alignment {
                        Alignment::None | Alignment::Left => layout::Alignment::Left,
                        Alignment::Center => layout::Alignment::Center,
                        Alignment::Right => layout::Alignment::Right,
                    };
                    let cell = if i == 0 { cell.bold() } else { cell };
                    let paragraph = Paragraph::new(cell)
                        .alignment(alignment)
                        .wrap(Wrap { trim: true });
                    (*width, paragraph)
                })
                .collect();
            Row::new(columns)
        })
        .collect()
}

fn column_widths(headers: &[Line], rows: &[Vec<Line>], width: u16) -> Option<Vec<u16>> {
    // Two columns of terminal space also leave room for a wide grapheme in a body cell.
    let mut widths: Vec<_> = headers.iter().map(|cell| cell.width().max(2)).collect();
    let gutters = usize::from(GUTTER) * widths.len().saturating_sub(1);
    let mut remaining = usize::from(width).checked_sub(gutters + widths.iter().sum::<usize>())?;
    let mut desired = widths.clone();
    for row in rows {
        for (max, cell) in desired.iter_mut().zip(row) {
            *max = (*max).max(cell.width());
        }
    }
    // Share spare width among columns that can use it before wrapping long cells.
    while remaining > 0 {
        let before = remaining;
        for (width, max) in widths.iter_mut().zip(&desired) {
            if *width < *max {
                *width += 1;
                remaining -= 1;
                if remaining == 0 {
                    break;
                }
            }
        }
        if before == remaining {
            break;
        }
    }
    Some(widths.into_iter().map(|w| w as u16).collect())
}

fn labeled_rows(rows: Vec<Vec<Line<'static>>>, width: u16) -> Vec<Row> {
    let mut rows = rows.into_iter();
    let headers = rows.next().unwrap_or_default();
    if rows.len() == 0 {
        return headers
            .into_iter()
            .map(|cell| Row::text(cell.bold(), width))
            .collect();
    }
    let mut result = Vec::new();
    for row in rows {
        if !result.is_empty() {
            result.push(Row::text(Line::default(), width));
        }
        for (i, (header, cell)) in headers.iter().zip(row).enumerate() {
            let mut line = if header.width() == 0 {
                Line::from(format!("Column {}", i + 1))
            } else {
                header.clone()
            };
            line.push_span(": ");
            line.spans.extend(cell.spans);
            result.push(Row::text(line, width));
        }
    }
    result
}
