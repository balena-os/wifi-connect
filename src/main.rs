use std::{process, path, thread, time};
use path::Path;

#[macro_use]
extern crate clap;
use clap::{Arg, App};

extern crate network_manager;
use network_manager::{manager, wifi, device, connection};

extern crate iron;
use iron::prelude::*;
use iron::{Iron, Request, Response, IronResult, status, typemap};

extern crate router;
use router::Router;

extern crate staticfile;
use staticfile::Static;

extern crate mount;
use mount::Mount;

extern crate serde_json;

extern crate persistent;
use persistent::State;

extern crate params;
use params::{Params, FromValue};

pub struct WiFiState {
    manager: manager::NetworkManager,
    device: device::Device,
    access_points: Vec<wifi::AccessPoint>,
    hotspot_connection: connection::Connection,
}

impl typemap::Key for WiFiState {
    type Value = WiFiState;
}

unsafe impl Send for WiFiState {}
unsafe impl Sync for WiFiState {}

fn main() {
    // TODO: error handling
    let matches = App::new("resin-wifi-connect")
        .version("0.1.0")
        .author("Joe Roberts <joe@resin.io>")
        .about("WiFi credentials configuration tool")
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

    let interface: Option<&str> = matches.value_of("interface");
    let ssid = matches.value_of("ssid").unwrap_or("resin-hotspot");
    let password: Option<&str> = matches.value_of("password");
    let timeout = matches.value_of("timeout").map_or(600, |v| v.parse::<u64>().unwrap());

    let manager = manager::new();
    let device = get_device(&manager, interface).unwrap();
    let access_points = get_access_points(&manager, &device).unwrap();
    let hotspot_connection = start_hotspot(&manager, &device, ssid, password).unwrap();

    let wifi_state = WiFiState {
        manager: manager,
        device: device,
        access_points: access_points,
        hotspot_connection: hotspot_connection,
    };

    // TODO: is this the best way to implement a timeout?
    thread::spawn(move || {
        start_server(wifi_state);
    });

    thread::sleep(time::Duration::new(timeout, 0));
    process::exit(1)
}

fn get_device(manager: &manager::NetworkManager,
              interface: Option<&str>)
              -> Result<device::Device, String> {
    let mut devices = device::list(&manager).unwrap();

    let index;
    if let Some(interface) = interface {
        index = devices.iter()
            .position(|ref d| d.device_type == device::DeviceType::WiFi && d.interface == interface)
            .unwrap();
    } else {
        index = devices.iter()
            .position(|ref d| d.device_type == device::DeviceType::WiFi)
            .unwrap();
    }

    Ok(devices.remove(index))
}

fn get_access_points(manager: &manager::NetworkManager,
                     device: &device::Device)
                     -> Result<Vec<wifi::AccessPoint>, String> {
    let mut access_points = wifi::scan(&manager, &device).unwrap();
    access_points.retain(|ap| ap.ssid != "");

    Ok(access_points)
}

fn start_hotspot(manager: &manager::NetworkManager,
                 device: &device::Device,
                 ssid: &str,
                 password: Option<&str>)
                 -> Result<connection::Connection, String> {
    connection::create_hotspot(&manager, &device, ssid, password.map(|p| p.to_owned()), 10)
}

fn stop_hotspot(manager: &manager::NetworkManager,
                connection: &connection::Connection)
                -> Result<(), String> {
    connection::disable(&manager, &mut connection.to_owned(), 10).unwrap();
    connection::delete(&manager, &connection)
}

fn start_server(wifi_state: WiFiState) {
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
    chain.link(State::<WiFiState>::both(wifi_state));

    Iron::new(chain).http("localhost:3000").unwrap();
}

fn ssid(req: &mut Request) -> IronResult<Response> {
    let lock = req.get_ref::<State<WiFiState>>().unwrap();
    let wifi_state = lock.read().unwrap();
    let payload = serde_json::to_string(&wifi_state.access_points
            .iter()
            .map(|ap| ap.ssid.clone())
            .collect::<Vec<String>>())
        .unwrap();

    Ok(Response::with((status::Ok, payload)))
}

fn connect(req: &mut Request) -> IronResult<Response> {
    let ssid;
    let password;
    {
        let map = req.get_ref::<Params>().unwrap();
        // TODO: is the best way to get the string from the value?
        ssid = <String as FromValue>::from_value(map.get("ssid").unwrap()).unwrap();
        password = <String as FromValue>::from_value(map.get("password").unwrap()).unwrap();
    }

    let lock = req.get_ref::<State<WiFiState>>().unwrap();
    let wifi_state = lock.read().unwrap();
    stop_hotspot(&wifi_state.manager, &wifi_state.hotspot_connection).unwrap();

    let index = wifi_state.access_points.iter().position(|ref ap| ap.ssid == ssid).unwrap();
    match connection::create(&wifi_state.manager,
                             &wifi_state.device,
                             &wifi_state.access_points[index],
                             &password,
                             10) {
        Ok(connection) => {
            // TODO: how do we respond and exit?
            Ok(Response::with(status::Ok))
            process::exit(0);
        }
        Err(error) => {
            // TODO: Delete connection - need connection::create to return connection object on
            // error
            // Ok(Response::with(status::
            process::exit(1);
        }
    }
}
