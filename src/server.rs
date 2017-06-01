use std::sync::mpsc::{Sender, Receiver};
use std::error::Error;
use std::fmt;

use serde_json;
use path::Path;
use iron::prelude::*;
use iron::{Iron, Request, Response, IronResult, status, typemap, IronError};
use router::Router;
use staticfile::Static;
use mount::Mount;
use persistent::State;
use params::{Params, FromValue};

use network::NetworkCommand;
use {shutdown, ShutdownResult};

struct RequestSharedState {
    server_rx: Receiver<Vec<String>>,
    network_tx: Sender<NetworkCommand>,
    shutdown_tx: Sender<ShutdownResult>,
}

impl typemap::Key for RequestSharedState {
    type Value = RequestSharedState;
}

unsafe impl Send for RequestSharedState {}
unsafe impl Sync for RequestSharedState {}

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &*self.0
    }
}

macro_rules! get_request_ref {
    ($req:ident, $ty:ty, $err:expr) => (
        match $req.get_ref::<$ty>() {
            Ok(val) => val,
            Err(err) => {
                error!($err);
                return Err(IronError::new(err, status::InternalServerError));
            }
        }
    )
}

macro_rules! get_param {
    ($params:ident, $param:expr, $ty:ty) => (
        match $params.get($param) {
            Some(value) => {
                match <$ty as FromValue>::from_value(value) {
                    Some(converted) => converted,
                    None => {
                        let err = format!("Unexpected type for '{}'", $param);
                        error!("{}", err);
                        return Err(IronError::new(StringError(err), status::InternalServerError));
                    }
                }
            },
            None => {
                let err = format!("'{}' not found in request params: {:?}", $param, $params);
                error!("{}", err);
                return Err(IronError::new(StringError(err), status::InternalServerError));
            }
        }
    )
}

macro_rules! get_request_state {
    ($req:ident) => (
        get_request_ref!(
            $req,
            State<RequestSharedState>,
            "Getting reference to request shared state failed"
        ).read().unwrap()
    )
}

macro_rules! shutdown_with_error {
    ($state:ident, $desc:expr) => (
        {
            shutdown(&$state.shutdown_tx, $desc.clone());
            return Err(IronError::new(StringError($desc), status::InternalServerError));
        }
    )
}

pub fn start_server(
    server_rx: Receiver<Vec<String>>,
    network_tx: Sender<NetworkCommand>,
    shutdown_tx: Sender<ShutdownResult>,
) {
    let shutdown_tx_clone = shutdown_tx.clone();
    let request_state = RequestSharedState {
        server_rx: server_rx,
        network_tx: network_tx,
        shutdown_tx: shutdown_tx,
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

    let address = "0.0.0.0:80";

    info!("Starting HTTP server on {}", address);

    if let Err(e) = Iron::new(chain).http(address) {
        shutdown(
            &shutdown_tx_clone,
            format!("Cannot start HTTP server on '{}': {}", address, e.description()),
        );
    }
}

fn ssid(req: &mut Request) -> IronResult<Response> {
    info!("Incoming `ssid` request");

    let request_state = get_request_state!(req);

    if let Err(err) = request_state.network_tx.send(NetworkCommand::Activate) {
        shutdown_with_error!(
            request_state,
            format!("Sending NetworkCommand::Activate failed: {}", err.description())
        );
    }

    let access_points_ssids = match request_state.server_rx.recv() {
        Ok(ssids) => ssids,
        Err(err) => {
            shutdown_with_error!(
                request_state,
                format!("Receiving access points ssids failed: {}", err.description())
            )
        },
    };

    let access_points_json = match serde_json::to_string(&access_points_ssids) {
        Ok(json) => json,
        Err(err) => {
            shutdown_with_error!(
                request_state,
                format!("Receiving access points ssids failed: {}", err.description())
            )
        },
    };

    Ok(Response::with((status::Ok, access_points_json)))
}

fn connect(req: &mut Request) -> IronResult<Response> {
    let (ssid, password) = {
        let params = get_request_ref!(req, Params, "Getting request params failed");
        let ssid = get_param!(params, "ssid", String);
        let password = get_param!(params, "password", String);
        (ssid, password)
    };

    info!("Incoming `connect` to access point `{}` request", ssid);

    let request_state = get_request_state!(req);

    let command = NetworkCommand::Connect {
        ssid: ssid,
        password: password,
    };

    if let Err(err) = request_state.network_tx.send(command) {
        shutdown_with_error!(
            request_state,
            format!("Sending NetworkCommand::Connect failed: {}", err.description())
        );
    }

    Ok(Response::with(status::Ok))
}
