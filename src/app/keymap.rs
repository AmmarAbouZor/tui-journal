use std::fmt::{Display, Formatter};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::ui::UICommand;

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
pub struct Input {
    pub key_code: KeyCode,
    pub modifiers: KeyModifiers,
    pub key_event: KeyEvent,
}

impl Input {
    pub fn new(key_code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            key_code,
            modifiers,
            key_event: KeyEvent::new(key_code, modifiers),
        }
    }
}

impl From<&KeyEvent> for Input {
    fn from(key_event: &KeyEvent) -> Self {
        Self {
            key_code: key_event.code,
            modifiers: key_event.modifiers,
            key_event: *key_event,
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
        // Char '?' isn't recognized on windows
        #[cfg(not(target_os = "windows"))]
        Keymap::new(
            Input::new(KeyCode::Char('?'), KeyModifiers::NONE),
            UICommand::ShowHelp,
        ),
        #[cfg(target_os = "windows")]
        Keymap::new(
            Input::new(KeyCode::Char('h'), KeyModifiers::NONE),
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
        Keymap::new(
            Input::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            UICommand::ToggleFullScreenMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
            UICommand::CycleTagFilter,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('u'), KeyModifiers::NONE),
            UICommand::Undo,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('U'), KeyModifiers::SHIFT),
            UICommand::Redo,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('b'), KeyModifiers::NONE),
            UICommand::ToggleViewMode,
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
            Input::new(KeyCode::Char('a'), KeyModifiers::NONE),
            UICommand::ShowFuzzyFind,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('F'), KeyModifiers::SHIFT),
            UICommand::ShowFuzzyFind,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('o'), KeyModifiers::NONE),
            UICommand::ShowSortOptions,
        ),
        Keymap::new(
            Input::new(KeyCode::Home, KeyModifiers::NONE),
            UICommand::GoToTopEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::End, KeyModifiers::NONE),
            UICommand::GoToBottomEntry,
        ),
        Keymap::new(
            Input::new(KeyCode::PageUp, KeyModifiers::NONE),
            UICommand::PageUpEntries,
        ),
        Keymap::new(
            Input::new(KeyCode::PageDown, KeyModifiers::NONE),
            UICommand::PageDownEntries,
        ),
        // Folder navigation (Right/l = enter, Left/h = back)
        Keymap::new(
            Input::new(KeyCode::Right, KeyModifiers::NONE),
            UICommand::FolderNavEnter,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('l'), KeyModifiers::NONE),
            UICommand::FolderNavEnter,
        ),
        Keymap::new(
            Input::new(KeyCode::Left, KeyModifiers::NONE),
            UICommand::FolderNavBack,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('h'), KeyModifiers::NONE),
            UICommand::FolderNavBack,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('r'), KeyModifiers::NONE),
            UICommand::RenameFolder,
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
            UICommand::BackEditorNormalMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            UICommand::BackEditorNormalMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('['), KeyModifiers::CONTROL),
            UICommand::BackEditorNormalMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('v'), KeyModifiers::NONE),
            UICommand::ToggleEditorVisualMode,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            UICommand::CutOsClipboard,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('v'), KeyModifiers::CONTROL),
            UICommand::CopyOsClipboard,
        ),
        Keymap::new(
            Input::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            UICommand::PasteOsClipboard,
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
        // Char '?' isn't recognized on windows
        #[cfg(not(target_os = "windows"))]
        Keymap::new(
            Input::new(KeyCode::Char('?'), KeyModifiers::NONE),
            UICommand::ShowHelp,
        ),
        #[cfg(target_os = "windows")]
        Keymap::new(
            Input::new(KeyCode::Char('h'), KeyModifiers::NONE),
            UICommand::ShowHelp,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyEvent;

    use super::*;

    #[test]
    fn display_plain_space() {
        assert_eq!(
            Input::new(KeyCode::Char(' '), KeyModifiers::NONE).to_string(),
            "<Space>"
        );
        assert_eq!(
            Input::new(KeyCode::Left, KeyModifiers::NONE).to_string(),
            "Left"
        );
    }

    #[test]
    fn display_with_modifiers() {
        let input = Input::new(
            KeyCode::Char('x'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT,
        );

        assert_eq!(input.to_string(), "<Ctrl-Shift-Alt-x>");
    }

    #[test]
    fn from_key_event() {
        let event = KeyEvent::new(KeyCode::PageDown, KeyModifiers::ALT);

        let input = Input::from(&event);

        assert_eq!(input.key_code, KeyCode::PageDown);
        assert_eq!(input.modifiers, KeyModifiers::ALT);
    }

    #[test]
    fn global_bindings_include_undo_redo() {
        let keymaps = get_global_keymaps();

        assert!(keymaps.iter().any(|keymap| {
            keymap.key == Input::new(KeyCode::Char('u'), KeyModifiers::NONE)
                && keymap.command == UICommand::Undo
        }));
        assert!(keymaps.iter().any(|keymap| {
            keymap.key == Input::new(KeyCode::Char('U'), KeyModifiers::SHIFT)
                && keymap.command == UICommand::Redo
        }));
    }
}
