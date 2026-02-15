#![deny(clippy::disallowed_methods)]
#![deny(clippy::disallowed_macros)]

mod cli;
mod hotkey;
mod paths;
mod taskbar;
mod tray;

use crate::cli::{Cli, Command, HotkeyCommand};
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (rev ",
    env!("GIT_REVISION"),
    ")"
);

pub fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let cli: Cli = figue::Driver::new(
        figue::builder::<Cli>()
            .expect("schema should be valid")
            .cli(|c| c)
            .help(|h| h.version(VERSION))
            .build(),
    )
    .run()
    .unwrap();

    init_tracing(cli.global.debug)?;

    let command = cli.command.unwrap_or(Command::Run);

    match command {
        Command::Run => tray::run_tray(VERSION),
        Command::Toggle => {
            let enabled = taskbar::toggle_taskbar_auto_hide()?;
            println!("taskbar auto-hide: {}", if enabled { "enabled" } else { "disabled" });
            Ok(())
        }
        Command::Status => {
            let enabled = taskbar::is_taskbar_auto_hide_enabled()?;
            println!("taskbar auto-hide: {}", if enabled { "enabled" } else { "disabled" });
            Ok(())
        }
        Command::Home => {
            let home = paths::app_home()?;
            home.ensure_dir()?;
            println!("{}", home.path().display());
            Ok(())
        }
        Command::Cache => {
            let cache = paths::cache_home()?;
            cache.ensure_dir()?;
            println!("{}", cache.path().display());
            Ok(())
        }
        Command::Hotkey(args) => match args.command {
            HotkeyCommand::Set { expression } => {
                let hotkey = hotkey::save_hotkey_expression(&expression)?;
                println!("{}", hotkey.expression);
                Ok(())
            }
            HotkeyCommand::Show => {
                let hotkey = hotkey::load_hotkey()?;
                println!("{}", hotkey.expression);
                Ok(())
            }
        },
    }
}

fn init_tracing(debug: bool) -> eyre::Result<()> {
    let level = if debug {
        tracing::level_filters::LevelFilter::DEBUG
    } else {
        tracing::level_filters::LevelFilter::INFO
    };

    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(true)
        .with_max_level(level)
        .try_init()
        .map_err(|error| eyre::eyre!("Failed to initialize logging: {error}"))?;

    Ok(())
}
