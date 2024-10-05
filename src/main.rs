// #![deny(warnings)]

use cfg::{Cfg, Commands};
use clap::Parser;
use ruc::*;

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
    }
}
