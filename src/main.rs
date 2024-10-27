#![deny(warnings)]

use cfg::{Cfg, Commands};
use chaindev::beacon_based::common::BASE_DIR;
use clap::{crate_name, CommandFactory, Parser};
use clap_complete::{
    generate,
    shells::{Bash, Zsh},
};
use ruc::*;
use std::{fs, io};

mod cfg;
mod common;
mod ddev;
mod dev;
mod pos;

fn main() {
    let config = Cfg::parse();

    pnk!(vsdb::vsdb_set_base_dir(format!(
        "{}/.vsdb",
        BASE_DIR.as_str()
    )));

    let err_mgmt = |e: Box<dyn RucError>, mark: &str| {
        let e = e.to_string();
        let err = e.trim_start().trim_end();
        if 24 < err.lines().count() {
            let p = format!("/tmp/err.nbnet.{mark}.{}", datetime!().replace(" ", "_"));
            pnk!(fs::write(&p, err));
            eprintln!(
                "\x1b[0;31mWARNING\x1b[0m: err occur!\nThe err log is located at: {}",
                p
            );
        } else {
            eprintln!("{err}");
        }
    };

    match config.commands {
        Commands::Dev(cfg) => {
            if let Err(e) = dev::EnvCfg::from(cfg).exec() {
                err_mgmt(e, "dev");
            }
        }
        Commands::DDev(cfg) => {
            if let Err(e) = ddev::EnvCfg::from(cfg).exec() {
                err_mgmt(e, "d_dev");
            }
        }
        Commands::Deposit(cfg) => {
            let future = pos::deposit::deposit(
                &cfg.rpc_endpoint,
                &cfg.deposit_contract_addr,
                &cfg.deposit_data_json_path,
                &cfg.wallet_signkey_path,
            );
            if let Err(e) = sb::runtime::Builder::new_current_thread()
                .enable_time()
                .enable_io()
                .build()
                .unwrap()
                .block_on(future)
            {
                err_mgmt(e, "deposit");
            }
        }
        Commands::NewMnemonic => {
            println!("\n{}\n", pos::mnemonic::create_mnemonic_words())
        }
        Commands::GenZshCompletions => {
            generate(Zsh, &mut Cfg::command(), crate_name!(), &mut io::stdout());
        }
        Commands::GenBashCompletions => {
            generate(Bash, &mut Cfg::command(), crate_name!(), &mut io::stdout());
        }
    }
}
