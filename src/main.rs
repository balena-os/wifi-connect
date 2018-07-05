#![recursion_limit = "1024"]

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate clap;

extern crate env_logger;
extern crate iron;
extern crate iron_cors;
extern crate mount;
extern crate network_manager;
extern crate nix;
extern crate params;
extern crate persistent;
extern crate router;
extern crate serde_json;
extern crate staticfile;

mod errors;
mod config;
mod network;
mod server;
mod dnsmasq;
mod logger;
mod exit;
mod privileges;

use std::path;
use std::thread;
use std::sync::mpsc::channel;
use std::io::Write;
use std::process;

use errors::*;
use config::get_config;
use network::{init_networking, process_network_commands};
use exit::block_exit_signals;
use privileges::require_root;

fn main() {
    if let Err(ref e) = run() {
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "\x1B[1;31mError: {}\x1B[0m", e).expect(errmsg);

        for inner in e.iter().skip(1) {
            writeln!(stderr, "  caused by: {}", inner).expect(errmsg);
        }

        process::exit(exit_code(e));
    }
}

fn run() -> Result<()> {
    block_exit_signals()?;

    logger::init();

    let config = get_config();

    require_root()?;

    init_networking()?;

    let (exit_tx, exit_rx) = channel();

    thread::spawn(move || {
        process_network_commands(&config, &exit_tx);
    });

    match exit_rx.recv() {
        Ok(result) => if let Err(reason) = result {
            return Err(reason);
        },
        Err(e) => {
            return Err(e.into());
        },
    }

    Ok(())
}
