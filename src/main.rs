mod config;
mod network;
mod web;

use std::thread;

use crate::network::{create_channel, run_network_manager_loop};
use crate::web::run_web_loop;

#[tokio::main]
async fn main() {
    let (glib_sender, glib_receiver) = create_channel();

    thread::spawn(move || {
        run_network_manager_loop(glib_receiver);
    });

    run_web_loop(glib_sender).await;
}
