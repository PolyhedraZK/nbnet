// #![deny(warnings)]

use cfg::{Cfg, Commands};
use clap::{crate_name, CommandFactory, Parser};
use clap_complete::{
    generate,
    shells::{Bash, Zsh},
};
use ruc::*;
use std::io;

mod cfg;
mod common;
mod ddev;
mod dev;

fn main() {
    let config = Cfg::parse();

    match config.commands {
        Commands::Dev(cfg) => {
            pnk!(dev::EnvCfg::from(cfg).exec());
        }
        Commands::DDev(cfg) => {
            pnk!(ddev::EnvCfg::from(cfg).exec());
        }
        Commands::GenZshCompletions => {
            generate(Zsh, &mut Cfg::command(), crate_name!(), &mut io::stdout());
        }
        Commands::GenBashCompletions => {
            generate(Bash, &mut Cfg::command(), crate_name!(), &mut io::stdout());
        }
    }
}
