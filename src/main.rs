mod network;
mod opts;
mod web;

use std::thread;

use clap::Parser;

use crate::network::{create_channel, run_network_manager_loop};
use crate::opts::Opts;
use crate::web::run_web_loop;

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let (glib_sender, glib_receiver) = create_channel();

    thread::spawn(move || {
        run_network_manager_loop(opts, glib_receiver);
    });

    run_web_loop(glib_sender).await;
}
