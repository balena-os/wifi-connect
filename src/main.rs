mod network;
mod opts;
mod web;

use std::thread;

use anyhow::{Result, Context};

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

    let received = network_init_receiver
        .await
        .context("Failed to receive network initialization response");

    received
        .and_then(|r| r)
        .or_else(|e| Err(e).context("Failed to initialize network"))?;

    run_web_loop(glib_sender).await;

    Ok(())
}
