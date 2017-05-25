extern crate clap;
extern crate network_manager;
extern crate iron;
extern crate router;
extern crate staticfile;
extern crate mount;
extern crate serde_json;
extern crate persistent;
extern crate params;

mod cli;
mod network;
mod server;

use std::path;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;

use cli::parse_cli_options;
use network::process_network_commands;
use server::start_server;

fn main() {
    // TODO: error handling
    let cli_options = parse_cli_options();
    let timeout = cli_options.timeout;

    let (shutdown_tx, shutdown_rx) = channel();
    let (server_tx, server_rx) = channel();
    let (network_tx, network_rx) = channel();

    let shutdown_tx_clone = shutdown_tx.clone();

    thread::spawn(move || {
                      process_network_commands(cli_options,
                                               network_rx,
                                               server_tx,
                                               shutdown_tx_clone);
                  });

    thread::spawn(move || {
                      thread::sleep(Duration::from_secs(timeout));
                      shutdown_tx.send(()).unwrap();
                  });

    thread::spawn(move || { start_server(server_rx, network_tx); });

    shutdown_rx.recv().unwrap();
}
