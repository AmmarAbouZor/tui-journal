use crate::{
    app::{commands::UICommand, App, UIComponents},
    data::DataProvider,
};

pub(crate) fn execute_command<D: DataProvider>(
    command: UICommand,
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> anyhow::Result<()> {
    match command {
        UICommand::SelectedPrevEntry => {}
        UICommand::SelectedNextEntry => {}
        UICommand::CreateEntry => {}
        UICommand::DeleteCurrentEntry => {}
        UICommand::StartEditCurrentEntry => {}
        _ => unreachable!("{:?} is not implemented for entries list", command),
    }

    Ok(())
}
