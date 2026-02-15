use facet::Facet;
use figue::{self as args, FigueBuiltins};

#[derive(Facet, Debug)]
pub struct Cli {
    #[facet(flatten)]
    pub global: GlobalArgs,

    #[facet(flatten)]
    pub builtins: FigueBuiltins,

    #[facet(args::subcommand, default)]
    pub command: Command,
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
    Hotkey {
        #[facet(default, args::positional)]
        expression: Option<String>,
    },
}
