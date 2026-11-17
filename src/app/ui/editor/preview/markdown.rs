use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
};

use super::{Row, table};

pub fn layout(source: &str, width: u16) -> Vec<Row> {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let mut events = Parser::new_ext(source, options);
    let mut body = Body::default();
    while let Some(event) = events.next() {
        // Tight list items can start another block without ending a paragraph.
        // Leave a pending bullet in place when the block opens the item.
        if !body.inline.line.spans.is_empty()
            && matches!(
                event,
                Event::Start(
                    Tag::Heading { .. }
                        | Tag::CodeBlock(_)
                        | Tag::BlockQuote(_)
                        | Tag::HtmlBlock
                        | Tag::Paragraph
                ) | Event::Rule
            )
        {
            body.finish_line(width);
        }
        match event {
            Event::Start(Tag::Table(alignments)) => {
                body.flush(width);
                body.start_block(width);
                body.rows
                    .extend(table::layout(&mut events, &alignments, width));
                body.gap = true;
            }
            Event::Start(Tag::Heading { .. }) => {
                body.inline.push_style(Style::new().bold().underlined())
            }
            Event::Start(Tag::CodeBlock(_)) => body.inline.push_style(Style::new().cyan()),
            Event::End(TagEnd::Heading(_) | TagEnd::CodeBlock) => {
                body.inline.styles.pop();
                body.finish_block(width);
            }
            Event::End(TagEnd::Paragraph | TagEnd::HtmlBlock) => body.finish_block(width),
            Event::Start(Tag::List(start)) => {
                body.flush(width);
                body.lists.push(start);
            }
            Event::Start(Tag::Item) => {
                body.marker = Some(match body.lists.last_mut() {
                    Some(Some(number)) => {
                        let marker = format!("{number}. ");
                        *number += 1;
                        marker
                    }
                    _ => "• ".into(),
                });
            }
            Event::End(TagEnd::Item) => body.flush(width),
            Event::End(TagEnd::List(_)) => {
                body.lists.pop();
                body.gap = body.lists.is_empty();
            }
            Event::Start(Tag::BlockQuote(_)) => body.quote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => {
                body.finish_block(width);
                body.quote_depth -= 1;
            }
            Event::Text(text) | Event::Html(text) => {
                for part in text.split_inclusive('\n') {
                    if let Some(line) = part.strip_suffix('\n') {
                        body.inline.text(line.to_owned());
                        body.finish_line(width);
                    } else {
                        body.inline.text(part.to_owned());
                    }
                }
            }
            Event::Rule => {
                body.inline.text("─".repeat(usize::from(width)));
                body.finish_block(width);
            }
            Event::SoftBreak | Event::HardBreak => body.finish_line(width),
            event => body.inline.event(event),
        }
    }
    body.flush(width);
    body.rows
}

#[derive(Default)]
struct Body {
    rows: Vec<Row>,
    inline: Inlines,
    lists: Vec<Option<u64>>,
    marker: Option<String>,
    quote_depth: usize,
    gap: bool,
}

impl Body {
    fn start_block(&mut self, width: u16) {
        if self.gap && !self.rows.is_empty() {
            self.rows.push(Row::text(Line::default(), width));
        }
        self.gap = false;
    }

    fn finish_line(&mut self, width: u16) {
        self.start_block(width);
        let mut line = Line::default();
        if self.quote_depth != 0 {
            line.push_span("│ ".repeat(self.quote_depth));
        }
        if !self.lists.is_empty() {
            line.push_span("  ".repeat(self.lists.len() - 1));
            line.push_span(self.marker.take().unwrap_or_else(|| "  ".into()));
        }
        line.spans.append(&mut self.inline.line.spans);
        self.rows.push(Row::text(line, width));
    }

    fn flush(&mut self, width: u16) {
        if !self.inline.line.spans.is_empty() || self.marker.is_some() {
            self.finish_line(width);
        }
    }

    fn finish_block(&mut self, width: u16) {
        self.flush(width);
        self.gap = true;
    }
}

#[derive(Default)]
pub struct Inlines {
    pub line: Line<'static>,
    styles: Vec<Style>,
    targets: Vec<Option<String>>,
}

impl Inlines {
    fn style(&self) -> Style {
        self.styles.last().copied().unwrap_or_default()
    }

    fn push_style(&mut self, style: Style) {
        self.styles.push(self.style().patch(style));
    }

    fn text(&mut self, text: String) {
        self.line.push_span(display_span(text, self.style()));
    }

    pub fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Text(s) | Event::InlineHtml(s) => self.text(s.into_string()),
            Event::Code(s) => self
                .line
                .push_span(display_span(s.into_string(), self.style().cyan())),
            Event::Start(Tag::Strong) => self.push_style(Style::new().bold()),
            Event::Start(Tag::Emphasis) => self.push_style(Style::new().italic()),
            Event::Start(Tag::Strikethrough) => {
                self.push_style(Style::new().add_modifier(Modifier::CROSSED_OUT))
            }
            Event::Start(
                Tag::Link {
                    dest_url,
                    link_type,
                    ..
                }
                | Tag::Image {
                    dest_url,
                    link_type,
                    ..
                },
            ) => {
                self.push_style(Style::new().underlined());
                let target = (!matches!(link_type, LinkType::Autolink | LinkType::Email))
                    .then(|| dest_url.into_string());
                self.targets.push(target);
            }
            Event::End(TagEnd::Link | TagEnd::Image) => {
                self.styles.pop();
                if let Some(Some(url)) = self.targets.pop() {
                    self.text(format!(" ({url})"));
                }
            }
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough) => {
                self.styles.pop();
            }
            Event::TaskListMarker(checked) => {
                self.text(if checked { "[x] " } else { "[ ] " }.into())
            }
            _ => {}
        }
    }
}

fn display_span(text: String, style: Style) -> Span<'static> {
    // Paragraph writes cell symbols directly to the terminal.
    let text = if text.contains(char::is_control) {
        text.replace(char::is_control, " ")
    } else {
        text
    };
    Span::styled(text, style)
}
