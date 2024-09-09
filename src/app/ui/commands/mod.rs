use backend::DataProvider;
use std::fmt::Debug;

use multi_select_cmd::*;

use super::{App, HandleInputReturnType, MsgBoxResult, UIComponents};

use editor_cmd::*;
use entries_list_cmd::*;
use global_cmd::*;

mod editor_cmd;
mod entries_list_cmd;
mod global_cmd;
mod multi_select_cmd;

type CmdResult = anyhow::Result<HandleInputReturnType>;

#[derive(Debug, Clone, Copy)]
pub enum ClipboardOperation {
    Copy,
    Cut,
    // I didn't use it yet but it's an option from clipboard operation and shouldn't be deleted
    #[allow(dead_code)]
    Paste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UICommand {
    Quit,
    ShowHelp,
    CycleFocusedControlForward,
    CycleFocusedControlBack,
    SelectedNextEntry,
    SelectedPrevEntry,
    CreateEntry,
    EditCurrentEntry,
    DeleteCurrentEntry,
    StartEditEntryContent,
    BackEditorNormalMode,
    SaveEntryContent,
    DiscardChangesEntryContent,
    ReloadAll,
    ExportEntryContent,
    EditInExternalEditor,
    EnterMultiSelectMode,
    LeaveMultiSelectMode,
    MulSelToggleSelected,
    MulSelSelectAll,
    MulSelSelectNone,
    MulSelInverSelection,
    MulSelDeleteEntries,
    MulSelExportEntries,
    ShowFilter,
    ResetFilter,
    ToggleTagFilter,
    ShowFuzzyFind,
    ToggleEditorVisualMode,
    ToggleFullScreenMode,
    CopyOsClipboard,
    CutOsClipboard,
    PasteOsClipboard,
    ShowSortOptions,
    GoToTopEntry,
    GoToBottomEntry,
    PageUpEntries,
    PageDownEntries,
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
}

impl CommandInfo {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
        }
    }
}

impl UICommand {
    pub fn get_info(&self) -> CommandInfo {
        match self {
            UICommand::Quit => CommandInfo::new("Exit", "Exit the program"),
            UICommand::ShowHelp => CommandInfo::new("Show help", "Show keybindings overview"),
            UICommand::CycleFocusedControlForward => {
                CommandInfo::new("Cycle focus forward", "Move focus to the next control")
            }
            UICommand::CycleFocusedControlBack => {
                CommandInfo::new("Cycle focus backward", "Move focus to the previous control")
            }
            UICommand::SelectedNextEntry => CommandInfo::new(
                "Select next journal",
                "Select next entry in the journals list",
            ),
            UICommand::SelectedPrevEntry => CommandInfo::new(
                "Select previous journal",
                "Select previous entry in the journal list",
            ),
            UICommand::CreateEntry => CommandInfo::new(
                "Create new journal",
                "Opens dialog to add a new journal entry",
            ),
            UICommand::EditCurrentEntry => CommandInfo::new(
                "Edit current journal",
                "Open entry dialog to edit current journal entry if any",
            ),
            UICommand::DeleteCurrentEntry => {
                CommandInfo::new("Delete journal", "Delete current journal entry if any")
            }
            UICommand::StartEditEntryContent => CommandInfo::new(
                "Edit journal content",
                "Start editing current journal entry content in editor",
            ),
            UICommand::BackEditorNormalMode => {
                CommandInfo::new("Back to Editor Normal Mode", "Exit editor special modes (insert, visual) and go back to normal mode")
            }
            UICommand::SaveEntryContent => {
                CommandInfo::new("Save", "Save changes on journal content")
            }
            UICommand::DiscardChangesEntryContent => {
                CommandInfo::new("Discard changes", "Discard changes on journal content")
            }
            UICommand::ReloadAll => CommandInfo::new("Reload all", "Reload all entries"),
            UICommand::ExportEntryContent => {
                CommandInfo::new("Export journal content", "Export current journal content")
            }
            UICommand::EditInExternalEditor => CommandInfo::new(
                "Edit in external editor",
                "Edit current journal content in external editor (The editor can be set in configurations file or via the environment variables VISUAL, EDITOR)",
            ),
            UICommand::EnterMultiSelectMode => CommandInfo::new(
                "Enter journals multi selection mode",
                "Enter multi selection mode for journals when journals list is in focus to work with multi journals at once",
            ),
            UICommand::LeaveMultiSelectMode => CommandInfo::new(
                "Leave journals multi selection mode",
                "Leave multi selection mode for journals and return to normal mode",
            ),
            UICommand::MulSelToggleSelected => CommandInfo::new(
                "Toggle selected",
                "Toggle if the current journal is selected in multi selection mode",
            ),
            UICommand::MulSelSelectAll => CommandInfo::new(
                "Select all journals",
                "Select all journals in multi selection mode",
            ),
            UICommand::MulSelSelectNone => CommandInfo::new(
                "Clear selection",
                "Clear journals selection in multi selection mode",
            ),
            UICommand::MulSelInverSelection => CommandInfo::new(
                "Invert selection",
                "Invert journals selection in multi selection mode",
            ),
            UICommand::MulSelDeleteEntries => CommandInfo::new(
                "Delete selection",
                "Delete selected journals in multi selection mode",
            ),
            UICommand::MulSelExportEntries => CommandInfo::new(
                "Export selection",
                "Export selected journals to a transfer JSON file, which can be imported to other back-end files",
            ),
            UICommand::ShowFilter => CommandInfo::new(
                "Open filter",
                "Open filter popup for journals",
            ),
            UICommand::ResetFilter => CommandInfo::new(
                "Reset filter",
                "Reset the applied filter on journals",
            ),
            UICommand::ToggleTagFilter => CommandInfo::new(
                "Toggle Tag Filter",
                "Cycle through the tag filters",
            ),
            UICommand::ShowFuzzyFind => CommandInfo::new(
                "Fuzzy find",
                "Open fuzzy find popup for journals",
            ),
            UICommand::ToggleEditorVisualMode => CommandInfo::new(
                "Toggle Editor Visual Mode",
                "Toggle Editor Visual(Select) Mode when editor is in focus",
            ),
            UICommand::ToggleFullScreenMode => CommandInfo::new(
                "Toggle Full Screen Mode",
                "Maximize the currently selected view",
            ),
            UICommand::CopyOsClipboard => CommandInfo::new(
                "Copy to OS clipboard",
                "Copy selection to operation system clipboard while in editor visual mode",
            ),
            UICommand::CutOsClipboard => CommandInfo::new(
                "Cut to OS clipboard",
                "Cut selection to operation system clipboard while in editor visual mode",
            ),
            UICommand::PasteOsClipboard => CommandInfo::new(
                "Paste OS clipboard Content",
                "Paste the operation system clipboard content to in the editor",
            ),
            UICommand::ShowSortOptions => CommandInfo::new(
                "Open sort options",
                "Open sort popup to set the sorting options of the journals",
            ),
            UICommand::GoToTopEntry => CommandInfo::new(
                "Go to top journal",
                "Go to the top entry in the journals' list",
            ),
            UICommand::GoToBottomEntry => CommandInfo::new(
                "Go to bottom journal",
                "Go to the bottom entry in the journals' list",
            ),
            UICommand::PageUpEntries => CommandInfo::new(
                "Page Up journals",
                "Go one page up in the journals' list",
            ),
            UICommand::PageDownEntries => CommandInfo::new(
                "Page Down journals",
                "Go one page down in the journals' list",
            ),
            UICommand::Undo => CommandInfo::new("Undo", "Undo the latest change on journals"),
            UICommand::Redo => CommandInfo::new("Redo", "Redo the latest change on journals"),

        }
    }

    pub async fn execute<'a, D: DataProvider>(
        &self,
        ui_components: &mut UIComponents<'a>,
        app: &mut App<D>,
    ) -> CmdResult {
        match self {
            UICommand::Quit => exec_quit(ui_components),
            UICommand::ShowHelp => exec_show_help(ui_components),
            UICommand::CycleFocusedControlForward => exec_cycle_forward(ui_components),
            UICommand::CycleFocusedControlBack => exec_cycle_backward(ui_components),
            UICommand::SelectedNextEntry => exec_select_next_entry(ui_components, app),
            UICommand::SelectedPrevEntry => exec_select_prev_entry(ui_components, app),
            UICommand::CreateEntry => exec_create_entry(ui_components, app),
            UICommand::EditCurrentEntry => exec_edit_current_entry(ui_components, app),
            UICommand::DeleteCurrentEntry => exec_delete_current_entry(ui_components, app),
            UICommand::StartEditEntryContent => exec_start_edit_content(ui_components),
            UICommand::BackEditorNormalMode => exec_back_editor_to_normal_mode(ui_components),
            UICommand::SaveEntryContent => exec_save_entry_content(ui_components, app).await,
            UICommand::DiscardChangesEntryContent => exec_discard_content(ui_components),
            UICommand::ReloadAll => exec_reload_all(ui_components, app).await,
            UICommand::ExportEntryContent => exec_export_entry_content(ui_components, app),
            UICommand::EditInExternalEditor => {
                exec_edit_in_external_editor(ui_components, app).await
            }
            UICommand::EnterMultiSelectMode => exec_enter_select_mode(ui_components),
            UICommand::LeaveMultiSelectMode => exec_leave_select_mode(ui_components, app),
            UICommand::MulSelToggleSelected => exec_toggle_selected(app),
            UICommand::MulSelSelectAll => exec_select_all(app),
            UICommand::MulSelSelectNone => exec_select_none(app),
            UICommand::MulSelInverSelection => exec_invert_selection(app),
            UICommand::MulSelDeleteEntries => exec_delete_selected_entries(ui_components, app),
            UICommand::MulSelExportEntries => exec_export_selected_entries(ui_components, app),
            UICommand::ShowFilter => exec_show_filter(ui_components, app),
            UICommand::ResetFilter => exec_reset_filter(app),
            UICommand::ToggleTagFilter => exec_toggle_tag_filter(app),
            UICommand::ShowFuzzyFind => exec_show_fuzzy_find(ui_components, app),
            UICommand::ToggleEditorVisualMode => exec_toggle_editor_visual_mode(ui_components),
            UICommand::ToggleFullScreenMode => exec_toggle_full_screen_mode(app),
            UICommand::CopyOsClipboard => exec_copy_os_clipboard(ui_components),
            UICommand::CutOsClipboard => exec_cut_os_clipboard(ui_components),
            UICommand::PasteOsClipboard => exec_paste_os_clipboard(ui_components),
            UICommand::ShowSortOptions => exec_show_sort_options(ui_components, app),
            cmd @ UICommand::GoToTopEntry => {
                check_unsaved_then_exec_cmd(*cmd, go_to_top_entry, ui_components, app)
            }
            cmd @ UICommand::GoToBottomEntry => {
                check_unsaved_then_exec_cmd(*cmd, go_to_bottom_entry, ui_components, app)
            }
            cmd @ UICommand::PageUpEntries => {
                check_unsaved_then_exec_cmd(*cmd, page_up_entries, ui_components, app)
            }
            cmd @ UICommand::PageDownEntries => {
                check_unsaved_then_exec_cmd(*cmd, page_down_entries, ui_components, app)
            }
            UICommand::Undo => exec_undo(ui_components, app).await,
            UICommand::Redo => exec_redo(ui_components, app).await,
        }
    }

    pub async fn continue_executing<'a, D: DataProvider>(
        &self,
        ui_components: &mut UIComponents<'a>,
        app: &mut App<D>,
        msg_box_result: MsgBoxResult,
    ) -> CmdResult {
        let not_implemented = || unreachable!("continue exec isn't implemented for {:?}", self);
        match self {
            UICommand::Quit => continue_quit(ui_components, app, msg_box_result).await,
            UICommand::ShowHelp => not_implemented(),
            UICommand::CycleFocusedControlForward => not_implemented(),
            UICommand::CycleFocusedControlBack => not_implemented(),
            UICommand::SelectedNextEntry => {
                continue_select_next_entry(ui_components, app, msg_box_result).await
            }
            UICommand::SelectedPrevEntry => {
                continue_select_prev_entry(ui_components, app, msg_box_result).await
            }
            UICommand::CreateEntry => {
                continue_create_entry(ui_components, app, msg_box_result).await
            }
            UICommand::EditCurrentEntry => {
                continue_edit_current_entry(ui_components, app, msg_box_result).await
            }
            UICommand::DeleteCurrentEntry => {
                continue_delete_current_entry(app, msg_box_result).await
            }
            UICommand::StartEditEntryContent => not_implemented(),
            UICommand::BackEditorNormalMode => not_implemented(),
            UICommand::SaveEntryContent => not_implemented(),
            UICommand::DiscardChangesEntryContent => {
                continue_discard_content(ui_components, app, msg_box_result)
            }
            UICommand::ReloadAll => continue_reload_all(ui_components, app, msg_box_result).await,
            UICommand::ExportEntryContent => {
                continue_export_entry_content(ui_components, app, msg_box_result).await
            }
            UICommand::EditInExternalEditor => {
                continue_edit_in_external_editor(ui_components, app, msg_box_result).await
            }
            UICommand::EnterMultiSelectMode => {
                continue_enter_select_mode(ui_components, app, msg_box_result).await
            }
            UICommand::LeaveMultiSelectMode => not_implemented(),
            UICommand::MulSelToggleSelected => not_implemented(),
            UICommand::MulSelSelectAll => not_implemented(),
            UICommand::MulSelSelectNone => not_implemented(),
            UICommand::MulSelInverSelection => not_implemented(),
            UICommand::MulSelDeleteEntries => {
                continue_delete_selected_entries(app, msg_box_result).await
            }
            UICommand::MulSelExportEntries => not_implemented(),
            UICommand::ShowFilter => continue_show_filter(ui_components, app, msg_box_result).await,
            UICommand::ResetFilter => not_implemented(),
            UICommand::ToggleTagFilter => not_implemented(),
            UICommand::ShowFuzzyFind => {
                continue_fuzzy_find(ui_components, app, msg_box_result).await
            }
            UICommand::ToggleEditorVisualMode => not_implemented(),
            UICommand::ToggleFullScreenMode => not_implemented(),
            UICommand::CopyOsClipboard => not_implemented(),
            UICommand::CutOsClipboard => not_implemented(),
            UICommand::PasteOsClipboard => not_implemented(),
            UICommand::ShowSortOptions => {
                continue_show_sort_options(ui_components, app, msg_box_result).await
            }
            UICommand::GoToTopEntry => {
                continue_cmd_after_check_unsaved(
                    go_to_top_entry,
                    ui_components,
                    app,
                    msg_box_result,
                )
                .await
            }
            UICommand::GoToBottomEntry => {
                continue_cmd_after_check_unsaved(
                    go_to_bottom_entry,
                    ui_components,
                    app,
                    msg_box_result,
                )
                .await
            }
            UICommand::PageUpEntries => {
                continue_cmd_after_check_unsaved(
                    page_up_entries,
                    ui_components,
                    app,
                    msg_box_result,
                )
                .await
            }
            UICommand::PageDownEntries => {
                continue_cmd_after_check_unsaved(
                    page_down_entries,
                    ui_components,
                    app,
                    msg_box_result,
                )
                .await
            }
            UICommand::Undo => continue_undo(ui_components, app, msg_box_result).await,
            UICommand::Redo => continue_redo(ui_components, app, msg_box_result).await,
        }
    }
}

/// Checks for unsaved and show dialog of there is any, other way it calls the `cmd_func`
pub fn check_unsaved_then_exec_cmd<D, F>(
    cmd: UICommand,
    cmd_func: F,
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult
where
    D: DataProvider,
    F: Fn(&mut UIComponents, &mut App<D>),
{
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(cmd));
    } else {
        cmd_func(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

/// Calls save entry content if wanted then calls `cmd_func`
pub async fn continue_cmd_after_check_unsaved<'a, D, F>(
    cmd_func: F,
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult
where
    D: DataProvider,
    F: Fn(&mut UIComponents, &mut App<D>),
{
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            cmd_func(ui_components, app);
        }
        MsgBoxResult::No => cmd_func(ui_components, app),
    }

    Ok(HandleInputReturnType::Handled)
}
