use std::process;

#[macro_use]
extern crate clap;
use clap::{Arg, App};

extern crate network_manager;
use network_manager::{manager, wifi, device, connection};

fn main() {
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
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Enable verbose output"))
        .get_matches();

    let interface: Option<&str> = matches.value_of("interface");
    let ssid = matches.value_of("ssid").unwrap_or("resin-hotspot");
    let password: Option<&str> = matches.value_of("password");
    let timeout = matches.value_of("timeout").map_or(600, |v| v.parse::<i32>().unwrap());
    let verbose = matches.is_present("verbose");

    if verbose {
        println!("Interface: {}, SSID: {}, Password: {}, Timeout: {}",
                 interface.unwrap_or("not set"),
                 ssid,
                 password.unwrap_or("not set"),
                 timeout);
    }

    let manager = manager::new();

    let mut devices = device::list(&manager).unwrap();
    let device_index = find_device(&devices, interface).unwrap();
    let device_ref = &mut devices[device_index];

    let access_points = wifi::scan(&manager, &device_ref).unwrap();
    if verbose {
        println!("Access points: {:?}", access_points)
    }

    let mut connections = connection::list(&manager).unwrap();
    match find_connection(&connections, ssid) {
        Some(connection_index) => {
            connection::enable(&manager, &mut connections[connection_index], 10).unwrap();
        }
        None => {
            connection::create_hotspot(&manager,
                                       &device_ref,
                                       ssid,
                                       password.map(|p| p.to_owned()),
                                       10)
                .unwrap();
        }
    }

    // Start server

    // Wait for credentials or timeout to elapse

    // Stop hotspot

    // Stop server

    // Add and activate credentials

    // Check connection

    // If not connected, delete credentials

    // Exit, 0 for success, 1 for failure
    process::exit(0)
}

fn find_device(devices: &Vec<device::Device>, interface: Option<&str>) -> Option<usize> {
    if let Some(interface) = interface {
        devices.iter()
            .position(|ref d| d.device_type == device::DeviceType::WiFi && d.interface == interface)
    } else {
        devices.iter()
            .position(|ref d| d.device_type == device::DeviceType::WiFi)
    }
}

fn find_connection(connections: &Vec<connection::Connection>, ssid: &str) -> Option<usize> {
    connections.iter()
        .position(|ref c| c.settings.ssid == ssid.to_owned())
}
