use std::convert::Infallible;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};

use warp::http::StatusCode;
use warp::Filter;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::exit::ExitResult;
use crate::network::{NetworkCommand, NetworkCommandResponse};

struct RequestSharedState {
    gateway: Ipv4Addr,
    server_rx: Receiver<NetworkCommandResponse>,
    network_tx: Sender<NetworkCommand>,
    exit_tx: Sender<ExitResult>,
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

type State = Arc<Mutex<RequestSharedState>>;

pub async fn start_server(
    gateway: Ipv4Addr,
    listening_port: u16,
    server_rx: Receiver<NetworkCommandResponse>,
    network_tx: Sender<NetworkCommand>,
    exit_tx: Sender<ExitResult>,
    ui_directory: &Path,
) {
    let exit_tx_clone = exit_tx.clone();
    let gateway_clone = gateway;

    let state = Arc::new(Mutex::new(RequestSharedState {
        gateway,
        server_rx,
        network_tx,
        exit_tx,
    }));

    let address = format!("{}:{}", gateway_clone, listening_port);
    info!("Starting HTTP server on {}", &address);

    let with_state = warp::any().map(move || state.clone());

    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file(ui_directory.join("index.html")));

    let static_ = warp::path("static").and(warp::fs::dir(ui_directory.join("static")));

    let networks = warp::path("networks").and(with_state).and_then(networks);

    let routes = index.or(static_).or(networks);

    warp::serve(routes)
        .run((gateway_clone, listening_port))
        .await;
}

async fn networks(state: State) -> std::result::Result<impl warp::Reply, Infallible> {
    let request_state = state.lock().await;

    if let Err(_e) = request_state.network_tx.send(NetworkCommand::Activate) {
        return Ok(warp::reply::json(&ErrorMessage {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "SendNetworkCommandActivate".into(),
        }));
        //return exit_with_error(&request_state, e, ErrorKind::SendNetworkCommandActivate);
    }

    let networks = match request_state.server_rx.recv() {
        Ok(result) => match result {
            NetworkCommandResponse::Networks(networks) => networks,
        },
        Err(_e) => {
            return Ok(warp::reply::json(&ErrorMessage {
                code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: "RecvAccessPointSSIDs".into(),
            }));
        }
    };

    Ok(warp::reply::json(&networks))
}

/*
fn connect(req: &mut Request) -> IronResult<Response> {
    let (ssid, identity, passphrase) = {
        let params = get_request_ref!(req, Params, "Getting request params failed");
        let ssid = get_param!(params, "ssid", String);
        let identity = get_param!(params, "identity", String);
        let passphrase = get_param!(params, "passphrase", String);
        (ssid, identity, passphrase)
    };

    debug!("Incoming `connect` to access point `{}` request", ssid);

    let request_state = get_request_state!(req);

    let command = NetworkCommand::Connect {
        ssid,
        identity,
        passphrase,
    };

    if let Err(e) = request_state.network_tx.send(command) {
        exit_with_error(&request_state, e, ErrorKind::SendNetworkCommandConnect)
    } else {
        Ok(Response::with(status::Ok))
    }
}
*/
