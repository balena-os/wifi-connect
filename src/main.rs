mod network;
mod opts;
mod web;

use std::thread;

use anyhow::{Context, Result};

use clap::Parser;

use tokio::sync::oneshot;

use crate::network::{create_channel, run_network_manager_loop};
use crate::opts::Opts;
use crate::web::run_web_loop;

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let (glib_sender, glib_receiver) = create_channel();

    let (network_init_sender, network_init_receiver) = oneshot::channel();

    thread::spawn(move || {
        run_network_manager_loop(opts, network_init_sender, glib_receiver);
    });

    receive_network_init_response(network_init_receiver).await?;

    run_web_loop(glib_sender).await;

    Ok(())
}

async fn receive_network_init_response(receiver: oneshot::Receiver<Result<()>>) -> Result<()> {
    let received = receiver
        .await
        .context("Failed to receive network initialization response");

    received
        .and_then(|r| r)
        .or_else(|e| Err(e).context("Failed to initialize network"))
}
