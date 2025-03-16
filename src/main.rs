use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use networks::rfkill;

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod tui;
mod widgets;
mod networks;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();

    rfkill::check()?;
    
    let mut app = App::new(args.tick_rate, args.frame_rate).await?;
    app.run().await?;
    Ok(())
}
