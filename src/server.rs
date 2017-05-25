use std::sync::mpsc::{Sender, Receiver};
use std::error::Error;

use serde_json;
use path::Path;
use iron::prelude::*;
use iron::{Iron, Request, Response, IronResult, status, typemap};
use router::Router;
use staticfile::Static;
use mount::Mount;
use persistent::State;
use params::{Params, FromValue};

use network::NetworkCommand;

struct RequestSharedState {
    server_rx: Receiver<Vec<String>>,
    network_tx: Sender<NetworkCommand>,
}

impl typemap::Key for RequestSharedState {
    type Value = RequestSharedState;
}

unsafe impl Send for RequestSharedState {}
unsafe impl Sync for RequestSharedState {}

pub fn start_server(server_rx: Receiver<Vec<String>>,
                    network_tx: Sender<NetworkCommand>,
                    shutdown_tx: Sender<Option<String>>) {
    let request_state = RequestSharedState {
        server_rx: server_rx,
        network_tx: network_tx,
    };

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

    let address = ":::80";

    if let Err(e) = Iron::new(chain).http(address) {
        let description = format!("Cannot start the web server on '{}': {}",
                                  address,
                                  e.description());
        let _ = shutdown_tx.send(Some(description));
    }
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
