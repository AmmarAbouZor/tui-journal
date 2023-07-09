use std::collections::HashMap;

use tui::{backend::Backend, layout::Rect, Frame};
use tui_textarea::TextArea;

use crate::app::keymap::Input;

pub struct FuzzFindPopup<'a> {
    text_box: TextArea<'a>,
    entries: HashMap<u32, String>,
}

pub enum FuzzFindReturn {
    KeepPopup,
    Close,
    SelectEntry(Option<u32>),
}

impl<'a> FuzzFindPopup<'a> {
    pub fn new(entries: HashMap<u32, String>) -> Self {
        let text_box = TextArea::default();
        Self { text_box, entries }
    }

    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        //TODO:
        todo!()
    }

    pub fn handle_input(&mut self, input: &Input) -> FuzzFindReturn {
        //TODO:
        todo!()
    }
}
