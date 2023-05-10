use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use anyhow::anyhow;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};

const LOG_MAX_SIZE: u64 = 1_000;

pub fn setup_logging(level: u8, path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = path.unwrap_or(get_default_path()?);

    let mut append = false;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
    } else if let Ok(size) = fs::metadata(&path).map(|data| data.len()) {
        append = size < LOG_MAX_SIZE;
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path)?;

    let log_level = u8_to_level(level);
    WriteLogger::init(log_level, Config::default(), file)?;

    log::trace!("Log initialized with level: {log_level}");

    Ok(())
}

pub fn get_default_path() -> anyhow::Result<PathBuf> {
    directories::BaseDirs::new()
        .map(|base_dir| {
            base_dir
                .cache_dir()
                .join("tui-journal")
                .join("tui-journal.log")
        })
        .ok_or_else(|| anyhow!("Log file path couldn't be retrieved"))
}

fn u8_to_level(num: u8) -> LevelFilter {
    match num {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}
