#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate network_manager;
extern crate iron;
extern crate router;
extern crate staticfile;
extern crate mount;
extern crate serde_json;
extern crate persistent;
extern crate params;

mod config;
mod network;
mod server;
mod dnsmasq;

use std::error::Error;
use std::path;
use std::thread;
use std::sync::mpsc::{channel, Sender};

use config::get_config;
use network::{process_network_commands, handle_existing_wifi_connections};

pub type ExitResult = Result<(), String>;

pub fn exit(exit_tx: &Sender<ExitResult>, error: String) {
    let _ = exit_tx.send(Err(error));
}

fn main() {
    env_logger::init().unwrap();

    let config = get_config();

    let (exit_tx, exit_rx) = channel();
    let (server_tx, server_rx) = channel();
    let (network_tx, network_rx) = channel();

    handle_existing_wifi_connections(config.clear);

    thread::spawn(move || {
        process_network_commands(&config, network_tx, network_rx, server_tx, server_rx, exit_tx);
    });

    match exit_rx.recv() {
        Ok(result) => {
            match result {
                Err(reason) => error!("{}", reason),
                Ok(_) => info!("Connection successfully established"),
            }
        },
        Err(e) => error!("Exit receiver error: {}", e.description()),
    }
}
