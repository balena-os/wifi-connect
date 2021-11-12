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

    let (initialized_sender, initialized_receiver) = oneshot::channel();

    thread::spawn(move || {
        run_network_manager_loop(opts, initialized_sender, glib_receiver);
    });

    receive_network_initialized(initialized_receiver).await?;

    run_web_loop(glib_sender).await;

    Ok(())
}

async fn receive_network_initialized(
    initialized_receiver: oneshot::Receiver<Result<()>>,
) -> Result<()> {
    let received = initialized_receiver
        .await
        .context("Failed to receive network initialization response");

    received
        .and_then(|r| r)
        .or_else(|e| Err(e).context("Failed to initialize network"))
}
