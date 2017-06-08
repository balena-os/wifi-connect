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

use std::error::Error;
use std::path;
use std::thread;
use std::sync::mpsc::{channel, Sender};

use config::get_config;
use network::{process_network_commands, handle_existing_wifi_connections};
use server::start_server;

pub type ShutdownResult = Result<(), String>;

pub fn shutdown(shutdown_tx: &Sender<ShutdownResult>, error: String) {
    let _ = shutdown_tx.send(Err(error));
}

fn main() {
    env_logger::init().unwrap();

    let config = get_config();

    let (shutdown_tx, shutdown_rx) = channel();
    let (server_tx, server_rx) = channel();
    let (network_tx, network_rx) = channel();

    let shutdown_tx_network = shutdown_tx.clone();

    handle_existing_wifi_connections(config.clear);

    thread::spawn(
        move || {
            process_network_commands(&config, &network_rx, &server_tx, &shutdown_tx_network);
        }
    );

    thread::spawn(move || { start_server(server_rx, network_tx, shutdown_tx); });

    match shutdown_rx.recv() {
        Ok(result) => {
            match result {
                Err(reason) => error!("{}", reason),
                Ok(_) => info!("Connection successfully established"),
            }
        },
        Err(e) => error!("Shutdown receiver error: {}", e.description()),
    }
}
