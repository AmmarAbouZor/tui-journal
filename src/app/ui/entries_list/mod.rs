use chrono::Datelike;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Margin,
    style::Style,
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
};

use backend::DataProvider;

use crate::app::App;
use crate::{app::keymap::Keymap, settings::DatumVisibility};

use super::{Styles, UICommand};

const LIST_INNER_MARGIN: usize = 5;

#[derive(Debug)]
pub struct EntriesList {
    pub state: ListState,
    is_active: bool,
    pub multi_select_mode: bool,
    /// Current folder path in folder navigation mode (empty = root).
    pub folder_path: Vec<String>,
    /// Selection state for the combined folder+entry list shown in folder nav mode.
    pub folder_list_state: ListState,
}

impl EntriesList {
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            is_active: false,
            multi_select_mode: false,
            folder_path: Vec::new(),
            folder_list_state: ListState::default(),
        }
    }

    fn render_list<D: DataProvider>(
        &mut self,
        frame: &mut Frame,
        app: &App<D>,
        area: Rect,
        styles: &Styles,
    ) {
        let jstyles = &styles.journals_list;

        let mut lines_count = 0;

        let items: Vec<ListItem> = app
            .get_active_entries()
            .map(|entry| {
                let highlight_selected =
                    self.multi_select_mode && app.selected_entries.contains(&entry.id);

                // *** Title ***
                let mut title = entry.title.to_string();

                if highlight_selected {
                    title.insert_str(0, "* ");
                }

                // Text wrapping
                let title_lines = textwrap::wrap(&title, area.width as usize - LIST_INNER_MARGIN);

                // tilte lines
                lines_count += title_lines.len();

                let title_style = match (self.is_active, highlight_selected) {
                    (_, true) => jstyles.title_selected,
                    (true, _) => jstyles.title_active,
                    (false, _) => jstyles.title_inactive,
                };

                let mut spans: Vec<Line> = title_lines
                    .iter()
                    .map(|line| Line::from(Span::styled(line.to_string(), title_style)))
                    .collect();

                // *** Date & Priority ***
                let date_priority_lines = match (app.settings.datum_visibility, entry.priority) {
                    (DatumVisibility::Show, Some(prio)) => {
                        let one_liner = format!(
                            "{},{},{} | Priority: {}",
                            entry.date.day(),
                            entry.date.month(),
                            entry.date.year(),
                            prio
                        );

                        if one_liner.len() > area.width as usize - LIST_INNER_MARGIN {
                            vec![
                                format!(
                                    "{},{},{}",
                                    entry.date.day(),
                                    entry.date.month(),
                                    entry.date.year()
                                ),
                                format!("Priority: {prio}"),
                            ]
                        } else {
                            vec![one_liner]
                        }
                    }
                    (DatumVisibility::Show, None) => {
                        vec![format!(
                            "{},{},{}",
                            entry.date.day(),
                            entry.date.month(),
                            entry.date.year()
                        )]
                    }
                    (DatumVisibility::Hide, None) => Vec::new(),
                    (DatumVisibility::EmptyLine, None) => vec![String::new()],
                    (_, Some(prio)) => {
                        vec![format!("Priority: {}", prio)]
                    }
                };

                let date_lines = date_priority_lines
                    .iter()
                    .map(|line| Line::from(Span::styled(line.to_string(), jstyles.date_priority)));
                spans.extend(date_lines);

                // date & priority lines
                lines_count += date_priority_lines.len();

                // *** Tags ***
                if !entry.tags.is_empty() {
                    const TAGS_SEPARATOR: &str = " | ";
                    let tags_default_style: Style = jstyles.tags_default.into();

                    let mut added_lines = 1;
                    spans.push(Line::default());

                    for tag in entry.tags.iter() {
                        let mut last_line = spans.last_mut().unwrap();
                        let allowd_width = area.width as usize - LIST_INNER_MARGIN;
                        if !last_line.spans.is_empty() {
                            if last_line.width() + TAGS_SEPARATOR.len() > allowd_width {
                                added_lines += 1;
                                spans.push(Line::default());
                                last_line = spans.last_mut().unwrap();
                            }
                            last_line.push_span(Span::styled(TAGS_SEPARATOR, tags_default_style))
                        }

                        let style = app
                            .get_color_for_tag(tag)
                            .map(|c| Style::default().bg(c.background).fg(c.foreground))
                            .unwrap_or(tags_default_style);
                        let span_to_add = Span::styled(tag.to_owned(), style);

                        if last_line.width() + tag.len() < allowd_width {
                            last_line.push_span(span_to_add);
                        } else {
                            added_lines += 1;
                            let line = Line::from(span_to_add);
                            spans.push(line);
                        }
                    }

                    lines_count += added_lines;
                }

                ListItem::new(spans)
            })
            .collect();

        let items_count = items.len();

        let highlight_style = if self.is_active {
            jstyles.highlight_active
        } else {
            jstyles.highlight_inactive
        };

        let list = List::new(items)
            .block(self.get_list_block(app.filter.is_some(), Some(items_count), styles))
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state);

        let lines_count = lines_count;

        if lines_count > area.height as usize - 2 {
            let avg_item_height = lines_count / items_count;

            self.render_scrollbar(
                frame,
                area,
                self.state.selected().unwrap_or(0),
                items_count,
                avg_item_height,
            );
        }
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Folder navigation view rendering
    // ────────────────────────────────────────────────────────────────────────────

    /// Render the folder navigation view.
    fn render_folder_view<D: DataProvider>(
        &mut self,
        frame: &mut Frame,
        app: &App<D>,
        area: Rect,
        styles: &Styles,
    ) {
        let jstyles = &styles.journals_list;

        // ── Layout: breadcrumb bar at top, then the list ─────────────────────
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        // ── Breadcrumb ────────────────────────────────────────────────────────
        let breadcrumb = self.build_breadcrumb();
        let bc_paragraph = Paragraph::new(breadcrumb)
            .style(jstyles.date_priority)
            .wrap(Wrap { trim: true });
        frame.render_widget(bc_paragraph, chunks[0]);

        // ── Build list items ──────────────────────────────────────────────────
        let tree = app.get_tag_tree();
        let node = tree.get_node(&self.folder_path);

        let mut items: Vec<ListItem> = Vec::new();
        let mut folder_count = 0;

        if let Some(node) = node {
            // Sub-folders first
            for name in node.subfolder_names() {
                items.push(self.make_folder_item(name, styles));
                folder_count += 1;
            }

            // Entries in this folder (no tags displayed)
            for entry in app.get_entries_in_folder(&self.folder_path) {
                items.push(self.make_entry_item_simple(entry, area.width as usize, styles, app));
            }
        }

        let items_count = items.len();

        let highlight_style = if self.is_active {
            jstyles.highlight_active
        } else {
            jstyles.highlight_inactive
        };

        let title = self.get_folder_view_block_title();
        let border_style = if self.is_active {
            jstyles.block_active
        } else {
            jstyles.block_inactive
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        let selected_info = if items_count > 0 {
            let sel = self.folder_list_state.selected().map(|v| v + 1).unwrap_or(0);
            Some(format!("{sel}/{items_count}"))
        } else {
            None
        };
        let block = if let Some(info) = selected_info {
            block.title_bottom(Line::from(info).right_aligned())
        } else {
            block
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.folder_list_state);

        if items_count > 0 && items_count > chunks[1].height as usize - 2 {
            let avg = 2_usize;
            self.render_scrollbar(
                frame,
                chunks[1],
                self.folder_list_state.selected().unwrap_or(0),
                items_count,
                avg,
            );
        }

        let _ = folder_count; // used implicitly for item construction ordering
    }

    fn build_breadcrumb(&self) -> String {
        if self.folder_path.is_empty() {
            "  /".to_owned()
        } else {
            format!("  / {}", self.folder_path.join(" / "))
        }
    }

    fn get_folder_view_block_title(&self) -> String {
        if self.folder_path.is_empty() {
            "Journals [Folder View]".to_owned()
        } else {
            format!("Journals [Folder View] › {}", self.folder_path.last().unwrap())
        }
    }

    fn make_folder_item<'a>(&self, name: &str, jstyles: &Styles) -> ListItem<'a> {
        let jstyles = &jstyles.journals_list;
        let label = format!("📁 {name}");
        ListItem::new(Line::from(Span::styled(
            label,
            if self.is_active {
                jstyles.title_active
            } else {
                jstyles.title_inactive
            },
        )))
    }

    fn make_entry_item_simple<'a, D: DataProvider>(
        &self,
        entry: &backend::Entry,
        width: usize,
        jstyles: &Styles,
        app: &App<D>,
    ) -> ListItem<'a> {
        let jstyles_inner = &jstyles.journals_list;
        let title_style = if self.is_active {
            jstyles_inner.title_active
        } else {
            jstyles_inner.title_inactive
        };

        let title_lines = textwrap::wrap(&entry.title, width.saturating_sub(LIST_INNER_MARGIN));
        let mut spans: Vec<Line> = title_lines
            .iter()
            .map(|line| Line::from(Span::styled(line.to_string(), title_style)))
            .collect();

        // Date/Priority (same logic as flat view, no tags)
        match (app.settings.datum_visibility, entry.priority) {
            (DatumVisibility::Show, Some(prio)) => {
                spans.push(Line::from(Span::styled(
                    format!(
                        "{},{},{} | Priority: {}",
                        entry.date.day(),
                        entry.date.month(),
                        entry.date.year(),
                        prio
                    ),
                    jstyles_inner.date_priority,
                )));
            }
            (DatumVisibility::Show, None) => {
                spans.push(Line::from(Span::styled(
                    format!(
                        "{},{},{}",
                        entry.date.day(),
                        entry.date.month(),
                        entry.date.year()
                    ),
                    jstyles_inner.date_priority,
                )));
            }
            (DatumVisibility::EmptyLine, None) => {
                spans.push(Line::default());
            }
            (_, Some(prio)) => {
                spans.push(Line::from(Span::styled(
                    format!("Priority: {prio}"),
                    jstyles_inner.date_priority,
                )));
            }
            _ => {}
        }

        let _ = app; // silence if unused by the match above
        ListItem::new(spans)
    }

    /// Return the name of the selected sub-folder (if the current selection is
    /// on a folder, not an entry).
    pub fn selected_folder_name<D: DataProvider>(&self, app: &App<D>) -> Option<String> {
        let sel = self.folder_list_state.selected()?;
        let tree = app.get_tag_tree();
        let node = tree.get_node(&self.folder_path)?;
        let folders: Vec<&str> = node.subfolder_names();
        folders.get(sel).map(|s| (*s).to_owned())
    }

    /// Return the entry id currently selected in folder nav mode (if the
    /// selection is on an entry row, not a folder).
    pub fn selected_folder_entry_id<D: DataProvider>(&self, app: &App<D>) -> Option<u32> {
        let sel = self.folder_list_state.selected()?;
        let tree = app.get_tag_tree();
        let node = tree.get_node(&self.folder_path)?;
        let folder_count = node.subfolders.len();
        if sel < folder_count {
            return None; // selection is on a folder
        }
        let entry_index = sel - folder_count;
        app.get_entries_in_folder(&self.folder_path)
            .nth(entry_index)
            .map(|e| e.id)
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Scroll / navigation helpers for folder mode
    // ────────────────────────────────────────────────────────────────────────────

    pub fn folder_nav_select_next<D: DataProvider>(&mut self, app: &App<D>) {
        let tree = app.get_tag_tree();
        let count = tree
            .get_node(&self.folder_path)
            .map(|n| {
                n.subfolders.len() + app.get_entries_in_folder(&self.folder_path).count()
            })
            .unwrap_or(0);
        if count == 0 {
            return;
        }
        let next = self
            .folder_list_state
            .selected()
            .map(|s| (s + 1).min(count - 1))
            .unwrap_or(0);
        self.folder_list_state.select(Some(next));
    }

    pub fn folder_nav_select_prev(&mut self) {
        let prev = self
            .folder_list_state
            .selected()
            .map(|s| s.saturating_sub(1))
            .unwrap_or(0);
        self.folder_list_state.select(Some(prev));
    }

    /// Sync the selection index and update the app's current entry.
    /// This should be called after any modification to `folder_path` or `folder_list_state`.
    pub fn sync_folder_nav_state<D: DataProvider>(&mut self, app: &mut App<D>) {
        if !app.state.folder_nav_mode {
            return;
        }

        let tree = app.get_tag_tree();
        let node = tree.get_node(&self.folder_path);

        let items_count = if let Some(node) = node {
            node.subfolders.len() + app.get_entries_in_folder(&self.folder_path).count()
        } else {
            0
        };

        if items_count > 0 {
            match self.folder_list_state.selected() {
                None => self.folder_list_state.select(Some(0)),
                Some(s) if s >= items_count => {
                    self.folder_list_state.select(Some(items_count - 1))
                }
                _ => {}
            }
        } else {
            self.folder_list_state.select(None);
        }

        // Always sync the current entry based on the (potentially clamped) selection.
        let entry_id = self.selected_folder_entry_id(app);
        app.current_entry_id = entry_id;
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Existing shared helpers
    // ────────────────────────────────────────────────────────────────────────────

    fn render_scrollbar(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        pos: usize,
        items_count: usize,
        avg_item_height: usize,
    ) {
        const VIEWPORT_ADJUST: u16 = 4;
        let viewport_len = (area.height / avg_item_height as u16).saturating_sub(VIEWPORT_ADJUST);

        let mut state = ScrollbarState::default()
            .content_length(items_count)
            .viewport_content_length(viewport_len as usize)
            .position(pos);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some(symbols::line::VERTICAL))
            .thumb_symbol(symbols::block::FULL);

        let scroll_area = area.inner(Margin {
            horizontal: 0,
            vertical: 1,
        });

        frame.render_stateful_widget(scrollbar, scroll_area, &mut state);
    }

    fn render_place_holder(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        list_keymaps: &[Keymap],
        has_filter: bool,
        styles: &Styles,
    ) {
        let keys_text: Vec<String> = list_keymaps
            .iter()
            .filter(|keymap| keymap.command == UICommand::CreateEntry)
            .map(|keymap| format!("'{}'", keymap.key))
            .collect();

        let place_holder_text = if self.multi_select_mode {
            String::from("\nNo entries to select")
        } else {
            format!("\n Use {} to create new entry ", keys_text.join(","))
        };

        let place_holder = Paragraph::new(place_holder_text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(self.get_list_block(has_filter, None, styles));

        frame.render_widget(place_holder, area);
    }

    fn get_list_block<'a>(
        &self,
        has_filter: bool,
        entries_len: Option<usize>,
        styles: &Styles,
    ) -> Block<'a> {
        let title = match (self.multi_select_mode, has_filter) {
            (true, true) => "Journals - Multi-Select - Filtered",
            (true, false) => "Journals - Multi-Select",
            (false, true) => "Journals - Filtered",
            (false, false) => "Journals",
        };

        let border_style = match (self.is_active, self.multi_select_mode) {
            (_, true) => styles.journals_list.block_multi_select,
            (true, _) => styles.journals_list.block_active,
            (false, _) => styles.journals_list.block_inactive,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        match (entries_len, self.state.selected().map(|v| v + 1)) {
            (Some(entries_len), Some(selected)) => {
                block.title_bottom(Line::from(format!("{selected}/{entries_len}")).right_aligned())
            }
            _ => block,
        }
    }

    pub fn render_widget<D: DataProvider>(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        app: &App<D>,
        list_keymaps: &[Keymap],
        styles: &Styles,
    ) {
        if app.state.folder_nav_mode {
            self.render_folder_view(frame, app, area, styles);
        } else if app.get_active_entries().next().is_none() {
            self.render_place_holder(frame, area, list_keymaps, app.filter.is_some(), styles);
        } else {
            self.render_list(frame, app, area, styles);
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}
