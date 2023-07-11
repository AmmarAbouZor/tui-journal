use std::fmt::{Display, Formatter};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::ui::UICommand;

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
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
            key_code: key_event.code,
            modifiers: key_event.modifiers,
        }
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut char_convert_tmp = [0; 4];
        let key_text = match self.key_code {
            KeyCode::Backspace => "<Backspace>",
            KeyCode::Enter => "Enter",
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
            KeyCode::Char(char) => {
                if char.is_whitespace() {
                    "<Space>"
                } else {
                    char.encode_utf8(&mut char_convert_tmp)
                }
            }
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

pub(crate) fn get_global_keymaps() -> Vec<Keymap> {
    vec![
        Keymap::new(
            Input::new(KeyCode::Char('q'), KeyModifiers::NONE),
            UICommand::Quit,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('?'), KeyModifiers::NONE),
            UICommand::ShowHelp,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
            UICommand::CycleFocusedControlForward,
        ),
        Keymap::new(
            Input::new(KeyCode::Tab, KeyModifiers::NONE),
            UICommand::CycleFocusedControlForward,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
            UICommand::CycleFocusedControlBack,
        ),
        Keymap::new(
            Input::new(KeyCode::BackTab, KeyModifiers::NONE),
            UICommand::CycleFocusedControlBack,
        ),
        Keymap::new(
            Input::new(KeyCode::Enter, KeyModifiers::NONE),
            UICommand::StartEditEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('m'), KeyModifiers::CONTROL),
            UICommand::StartEditEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('s'), KeyModifiers::NONE),
            UICommand::SaveEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
            UICommand::ReloadAll,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('y'), KeyModifiers::CONTROL),
            UICommand::EditInExternalEditor,
        ),
    ]
}

pub fn get_entries_list_keymaps() -> Vec<Keymap> {
    vec![
        Keymap::new(
            Input::new(KeyCode::Up, KeyModifiers::NONE),
            UICommand::SelectedPrevEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('k'), KeyModifiers::NONE),
            UICommand::SelectedPrevEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Down, KeyModifiers::NONE),
            UICommand::SelectedNextEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('j'), KeyModifiers::NONE),
            UICommand::SelectedNextEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('n'), KeyModifiers::NONE),
            UICommand::CreateEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('e'), KeyModifiers::NONE),
            UICommand::EditCurrentEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Delete, KeyModifiers::NONE),
            UICommand::DeleteCurrentEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('d'), KeyModifiers::NONE),
            UICommand::DeleteCurrentEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            UICommand::DeleteCurrentEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('>'), KeyModifiers::NONE),
            UICommand::ExportEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('y'), KeyModifiers::NONE),
            UICommand::EditInExternalEditor,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('v'), KeyModifiers::NONE),
            UICommand::EnterMultiSelectMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('f'), KeyModifiers::NONE),
            UICommand::ShowFilter,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('x'), KeyModifiers::NONE),
            UICommand::ResetFilter,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('u'), KeyModifiers::NONE),
            UICommand::ShowFuzzyFind,
        ),
    ]
}

pub(crate) fn get_editor_mode_keymaps() -> Vec<Keymap> {
    vec![
        Keymap::new(
            Input::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            UICommand::SaveEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            UICommand::DiscardChangesEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Esc, KeyModifiers::NONE),
            UICommand::FinishEditEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            UICommand::FinishEditEntryContent,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('['), KeyModifiers::CONTROL),
            UICommand::FinishEditEntryContent,
        ),
    ]
}

pub fn get_multi_select_keymaps() -> Vec<Keymap> {
    vec![
        Keymap::new(
            Input::new(KeyCode::Char('q'), KeyModifiers::NONE),
            UICommand::LeaveMultiSelectMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('v'), KeyModifiers::NONE),
            UICommand::LeaveMultiSelectMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Esc, KeyModifiers::NONE),
            UICommand::LeaveMultiSelectMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            UICommand::LeaveMultiSelectMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Up, KeyModifiers::NONE),
            UICommand::SelectedPrevEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('k'), KeyModifiers::NONE),
            UICommand::SelectedPrevEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Down, KeyModifiers::NONE),
            UICommand::SelectedNextEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('j'), KeyModifiers::NONE),
            UICommand::SelectedNextEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::Char(' '), KeyModifiers::NONE),
            UICommand::MulSelToggleSelected,
        ),
        Keymap::new(
            Input::new(KeyCode::Enter, KeyModifiers::NONE),
            UICommand::MulSelToggleSelected,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('m'), KeyModifiers::CONTROL),
            UICommand::MulSelToggleSelected,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('a'), KeyModifiers::NONE),
            UICommand::MulSelSelectAll,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('x'), KeyModifiers::NONE),
            UICommand::MulSelSelectNone,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('i'), KeyModifiers::NONE),
            UICommand::MulSelInverSelection,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('d'), KeyModifiers::NONE),
            UICommand::MulSelDeleteEntries,
        ),
        Keymap::new(
            Input::new(KeyCode::Delete, KeyModifiers::NONE),
            UICommand::MulSelDeleteEntries,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('>'), KeyModifiers::NONE),
            UICommand::MulSelExportEntries,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('?'), KeyModifiers::NONE),
            UICommand::ShowHelp,
        ),
    ]
}
