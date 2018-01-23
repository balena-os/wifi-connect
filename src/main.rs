#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate clap;
extern crate env_logger;
extern crate iron;
#[macro_use]
extern crate log;
extern crate mount;
extern crate network_manager;
extern crate params;
extern crate persistent;
extern crate router;
extern crate serde_json;
extern crate staticfile;

mod config;
mod network;
mod server;
mod dnsmasq;
mod logger;

use std::error::Error;
use std::path;
use std::thread;
use std::sync::mpsc::{channel, Sender};

use config::get_config;
use network::{init_networking, process_network_commands};

pub type ExitResult = Result<(), String>;

pub fn exit(exit_tx: &Sender<ExitResult>, error: String) {
    let _ = exit_tx.send(Err(error));
}

fn main() {
    logger::init();

    let config = get_config();

    init_networking();

    let (exit_tx, exit_rx) = channel();

    thread::spawn(move || {
        process_network_commands(&config, &exit_tx);
    });

    match exit_rx.recv() {
        Ok(result) => if let Err(reason) = result {
            error!("{}", reason);
        },
        Err(e) => error!("Exit receiver error: {}", e.description()),
    }
}
