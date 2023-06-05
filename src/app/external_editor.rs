use std::{env, ffi::OsStr, io, path::Path};

use anyhow::{anyhow, bail};

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use scopeguard::defer;
use tokio::process::Command;

use crate::settings::Settings;

const ENV_EDITOR_OPTIONS: [&str; 2] = ["VISUAL", "EDITOR"];

pub async fn open_editor(file_path: &Path, settings: &Settings) -> anyhow::Result<()> {
    if !file_path.exists() {
        bail!("file doesn't exist: {}", file_path.display());
    }

    let file_path = file_path.canonicalize()?;

    let editor_raw = settings
        .external_editor
        .as_ref()
        .cloned()
        .or_else(|| env::var(ENV_EDITOR_OPTIONS[0]).ok())
        .or_else(|| env::var(ENV_EDITOR_OPTIONS[1]).ok())
        .unwrap_or(String::from("vi"));

    if editor_raw.is_empty() {
        bail!(
            "The Editor in configuration and environmental variables is empty: {}",
            ENV_EDITOR_OPTIONS.join(" - ")
        );
    }

    let mut editor_chars = editor_raw.chars().peekable();

    let start_char = editor_chars
        .peek()
        .expect("Editor name can't be empty")
        .to_owned();

    let editor_cmd: String = match start_char {
        '\"' => editor_chars
            .by_ref()
            .skip(1)
            .take_while(|&c| c != '\"')
            .collect(),
        _ => editor_chars.by_ref().take_while(|&c| c != ' ').collect(),
    };

    let rest_args: String = editor_chars.collect();
    let mut args: Vec<&OsStr> = rest_args.split_whitespace().map(OsStr::new).collect();

    args.push(file_path.as_os_str());

    io::stdout().execute(LeaveAlternateScreen)?;
    defer! {
        io::stdout().execute(EnterAlternateScreen).unwrap();
    }

    Command::new(editor_cmd.clone())
        .args(args)
        .status()
        .await
        .map_err(|err| {
            anyhow!(
                "Error while openning the editor. Editor command: '{}'. Error: {}",
                editor_cmd,
                err
            )
        })?;

    Ok(())
}
