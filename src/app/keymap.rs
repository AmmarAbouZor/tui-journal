use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::commands::UICommand;

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
pub struct Input {
    pub key_code: KeyCode,
    pub modmodifiers: KeyModifiers,
}

impl Input {
    pub fn new(key_code: KeyCode, modmodifiers: KeyModifiers) -> Self {
        Self {
            key_code,
            modmodifiers,
        }
    }
}

impl From<&KeyEvent> for Input {
    fn from(key_event: &KeyEvent) -> Self {
        Self {
            key_code: key_event.code.clone(),
            modmodifiers: key_event.modifiers,
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
