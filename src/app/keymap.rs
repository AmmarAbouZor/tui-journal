use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq)]
pub struct Input {
    key_code: KeyCode,
    modmodifiers: KeyModifiers,
}

impl From<&KeyEvent> for Input {
    fn from(key_event: &KeyEvent) -> Self {
        Self {
            key_code: key_event.code.clone(),
            modmodifiers: key_event.modifiers,
        }
    }
}
