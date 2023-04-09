use std::fmt::{Display, Formatter};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::commands::UICommand;

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
pub struct Input {
    pub key_code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Input {
    pub fn new(key_code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            key_code,
            modifiers,
        }
    }
}

impl From<&KeyEvent> for Input {
    fn from(key_event: &KeyEvent) -> Self {
        Self {
            key_code: key_event.code.clone(),
            modifiers: key_event.modifiers,
        }
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut char_convert_tmp = [0; 4];
        let key_text = match self.key_code {
            KeyCode::Backspace => "<Backspace>",
            KeyCode::Enter => "<Return>",
            KeyCode::Left => "Left",
            KeyCode::Right => "Right",
            KeyCode::Up => "Up",
            KeyCode::Down => "Down",
            KeyCode::Home => "Home",
            KeyCode::End => "End",
            KeyCode::PageUp => "PageUp",
            KeyCode::PageDown => "PageDown",
            KeyCode::Tab => "Tab",
            KeyCode::BackTab => "BackTab",
            KeyCode::Delete => "Delete",
            KeyCode::Insert => "Isnert",
            KeyCode::F(_) => "F",
            KeyCode::Char(char) => char.encode_utf8(&mut char_convert_tmp),
            KeyCode::Null => "Null",
            KeyCode::Esc => "Esc",
            _ => panic!("{:?} is not implemented", self.key_code),
        };

        if self.modifiers.is_empty() {
            write!(f, "{key_text}")
        } else {
            let mut modifier_text = String::from("<");
            if self.modifiers.contains(KeyModifiers::CONTROL) {
                modifier_text.push_str("Ctrl-");
            }
            if self.modifiers.contains(KeyModifiers::SHIFT) {
                modifier_text.push_str("Shift-");
            }
            if self.modifiers.contains(KeyModifiers::ALT) {
                modifier_text.push_str("Alt-");
            }

            write!(f, "{modifier_text}{key_text}>")
        }
    }
}

#[derive(Debug)]
pub struct Keymap {
    pub key: Input,
    pub command: UICommand,
}

impl Keymap {
    pub fn new(key: Input, command: UICommand) -> Self {
        Self { key, command }
    }
}
