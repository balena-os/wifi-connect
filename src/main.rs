extern crate clap;
extern crate network_manager;
extern crate iron;
extern crate router;
extern crate staticfile;
extern crate mount;
extern crate serde_json;
extern crate persistent;
extern crate params;

use std::path;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

use path::Path;
use clap::{Arg, App};
use iron::prelude::*;
use iron::{Iron, Request, Response, IronResult, status, typemap};
use router::Router;
use staticfile::Static;
use mount::Mount;
use persistent::State;
use params::{Params, FromValue};

use network_manager::{NetworkManager, Device, DeviceType, Connection, AccessPoint};


enum NetworkCommand {
    Activate,
    Connect { ssid: String, password: String },
}

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
    let matches = App::new("resin-wifi-connect")
        .version("0.1.0")
        .author("Joe Roberts <joe@resin.io>")
        .about("Wi-Fi credentials configuration tool")
        .arg(Arg::with_name("interface")
                 .short("i")
                 .long("interface")
                 .value_name("INTERFACE")
                 .help("Hotspot interface")
                 .takes_value(true))
        .arg(Arg::with_name("ssid")
                 .short("s")
                 .long("ssid")
                 .value_name("SSID")
                 .help("Hotspot SSID")
                 .takes_value(true))
        .arg(Arg::with_name("password")
                 .short("p")
                 .long("password")
                 .value_name("PASSWORD")
                 .help("Hotspot password ")
                 .takes_value(true))
        .arg(Arg::with_name("timeout")
                 .short("t")
                 .long("timeout")
                 .value_name("TIMEOUT")
                 .help("Hotspot timeout (seconds)")
                 .takes_value(true))
        .get_matches();

    let interface: Option<String> = matches.value_of("interface").map(String::from);
    let ssid: String = matches
        .value_of("ssid")
        .unwrap_or("resin-hotspot")
        .to_string();
    let password: Option<String> = matches.value_of("password").map(String::from);
    let timeout = matches
        .value_of("timeout")
        .map_or(600, |v| v.parse::<u64>().unwrap());

    let (shutdown_tx, shutdown_rx) = channel();
    let (server_tx, server_rx) = channel();
    let (network_tx, network_rx) = channel();

    let request_state = RequestSharedState {
        server_rx: server_rx,
        network_tx: network_tx,
    };

    let shutdown_tx_clone = shutdown_tx.clone();

    thread::spawn(move || {
                      process_network_commands(interface,
                                               ssid,
                                               password,
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

fn find_device(manager: &NetworkManager, interface: Option<String>) -> Result<Device, String> {
    if let Some(interface) = interface {
        let device = manager.get_device_by_interface(&interface)?;

        if *device.device_type() == DeviceType::WiFi {
            Ok(device)
        } else {
            Err(format!("Not a Wi-Fi device: {}", interface))
        }
    } else {
        let devices = manager.get_devices()?;

        let index = devices
            .iter()
            .position(|ref d| *d.device_type() == DeviceType::WiFi);

        if let Some(index) = index {
            Ok(devices[index].clone())
        } else {
            Err("Cannot find a Wi-Fi device".to_string())
        }
    }
}

fn get_access_points(device: &Device) -> Result<Vec<AccessPoint>, String> {
    let wifi_device = device.as_wifi_device().unwrap();
    let mut access_points = wifi_device.get_access_points()?;
    access_points.retain(|ap| ap.ssid().as_str().is_ok());
    Ok(access_points)
}

fn create_hotspot(device: &Device,
                  ssid: &str,
                  password: &Option<&str>)
                  -> Result<Connection, String> {
    let wifi_device = device.as_wifi_device().unwrap();
    let (hotspot_connection, _) = wifi_device.create_hotspot(&ssid as &str, *password)?;
    Ok(hotspot_connection)
}

fn stop_hotspot(connection: &Connection) -> Result<(), String> {
    connection.deactivate()?;
    connection.delete()?;
    thread::sleep(Duration::from_secs(1));
    Ok(())
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

fn process_network_commands(interface: Option<String>,
                            hotspot_ssid: String,
                            hotspot_password: Option<String>,
                            network_rx: Receiver<NetworkCommand>,
                            server_tx: Sender<Vec<String>>,
                            shutdown_tx: Sender<()>) {
    let manager = NetworkManager::new();
    let device = find_device(&manager, interface).unwrap();

    let mut access_points_option = Some(get_access_points(&device).unwrap());

    let hotspot_password = hotspot_password.as_ref().map(|p| p as &str);
    let mut hotspot_connection = None;

    loop {
        let command = network_rx.recv().unwrap();

        match command {
            NetworkCommand::Activate => {
                if let Some(ref connection) = hotspot_connection {
                    stop_hotspot(connection).unwrap();
                }

                let access_points = if let Some(access_points) = access_points_option {
                    access_points
                } else {
                    get_access_points(&device).unwrap()
                };

                let access_points_ssids = access_points
                    .iter()
                    .map(|ap| ap.ssid().as_str().unwrap().to_string())
                    .collect::<Vec<String>>();

                hotspot_connection =
                    Some(create_hotspot(&device, &hotspot_ssid, &hotspot_password).unwrap());

                access_points_option = None;

                server_tx.send(access_points_ssids).unwrap();
            }
            NetworkCommand::Connect { ssid, password } => {
                if let Some(ref connection) = hotspot_connection {
                    stop_hotspot(connection).unwrap();
                }
                hotspot_connection = None;

                let access_points = get_access_points(&device).unwrap();

                for access_point in access_points {
                    if let Ok(access_point_ssid) = access_point.ssid().as_str() {
                        if access_point_ssid == &ssid {
                            let wifi_device = device.as_wifi_device().unwrap();

                            wifi_device
                                .connect(&access_point, &password as &str)
                                .unwrap();

                            shutdown_tx.send(()).unwrap();
                        }
                    }
                }
            }
        }
    }
}
