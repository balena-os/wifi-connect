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

use std::path;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

use path::Path;
use iron::prelude::*;
use iron::{Iron, Request, Response, IronResult, status, typemap};
use router::Router;
use staticfile::Static;
use mount::Mount;
use persistent::State;
use params::{Params, FromValue};

use cli::parse_cli_options;
use network::{process_network_commands, NetworkCommand};

struct RequestSharedState {
    server_rx: Receiver<Vec<String>>,
    network_tx: Sender<NetworkCommand>,
}

impl typemap::Key for RequestSharedState {
    type Value = RequestSharedState;
}

unsafe impl Send for RequestSharedState {}
unsafe impl Sync for RequestSharedState {}

fn main() {
    // TODO: error handling
    let cli_options = parse_cli_options();
    let timeout = cli_options.timeout;

    let (shutdown_tx, shutdown_rx) = channel();
    let (server_tx, server_rx) = channel();
    let (network_tx, network_rx) = channel();

    let request_state = RequestSharedState {
        server_rx: server_rx,
        network_tx: network_tx,
    };

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

    thread::spawn(move || { start_server(request_state); });

    shutdown_rx.recv().unwrap();
}

fn start_server(request_state: RequestSharedState) {
    let mut router = Router::new();
    router.get("/", Static::new(Path::new("public")), "index");
    router.get("/ssid", ssid, "ssid");
    router.post("/connect", connect, "connect");

    let mut assets = Mount::new();
    assets.mount("/", router);
    assets.mount("/css", Static::new(Path::new("public/css")));
    assets.mount("/img", Static::new(Path::new("public/img")));
    assets.mount("/js", Static::new(Path::new("public/js")));

    let mut chain = Chain::new(assets);
    chain.link(State::<RequestSharedState>::both(request_state));

    Iron::new(chain).http("localhost:3000").unwrap();
}

fn ssid(req: &mut Request) -> IronResult<Response> {
    let lock = req.get_ref::<State<RequestSharedState>>().unwrap();
    let request_state = lock.read().unwrap();

    request_state
        .network_tx
        .send(NetworkCommand::Activate)
        .unwrap();

    let access_points_ssids = request_state.server_rx.recv().unwrap();

    let access_points_json = serde_json::to_string(&access_points_ssids).unwrap();

    Ok(Response::with((status::Ok, access_points_json)))
}

fn connect(req: &mut Request) -> IronResult<Response> {
    let (ssid, password) = {
        let map = req.get_ref::<Params>().unwrap();
        let ssid = <String as FromValue>::from_value(map.get("ssid").unwrap()).unwrap();
        let password = <String as FromValue>::from_value(map.get("password").unwrap()).unwrap();
        (ssid, password)
    };

    let lock = req.get_ref::<State<RequestSharedState>>().unwrap();
    let request_state = lock.read().unwrap();

    let command = NetworkCommand::Connect {
        ssid: ssid,
        password: password,
    };
    request_state.network_tx.send(command).unwrap();

    Ok(Response::with(status::Ok))
}
