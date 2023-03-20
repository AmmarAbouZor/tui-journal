use anyhow::{Ok, Result};

mod app;

fn main() -> Result<()> {
    app::run()?;

    Ok(())
}
