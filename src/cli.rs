use facet::Facet;
use figue::{self as args, FigueBuiltins};

#[derive(Facet, Debug)]
pub struct Cli {
    #[facet(flatten)]
    pub global: GlobalArgs,

    #[facet(flatten)]
    pub builtins: FigueBuiltins,

    #[facet(args::subcommand)]
    pub command: Option<Command>,
}

#[derive(Facet, Debug, Default)]
pub struct GlobalArgs {
    #[facet(args::named, default)]
    pub debug: bool,
}

#[derive(Facet, Debug, Default)]
#[repr(u8)]
pub enum Command {
    #[default]
    Run,
    Toggle,
    Status,
    Home,
    Cache,
    Hotkey(HotkeyArgs),
}

#[derive(Facet, Debug)]
pub struct HotkeyArgs {
    #[facet(args::subcommand, default)]
    pub command: HotkeyCommand,
}

#[derive(Facet, Debug, Default)]
#[repr(u8)]
pub enum HotkeyCommand {
    #[default]
    Show,
    Set {
        #[facet(args::positional)]
        expression: String,
    },
}
