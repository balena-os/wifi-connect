use std::process;

#[macro_use]
extern crate clap;
use clap::{Arg, App};

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

    let interface = matches.value_of("interface").unwrap_or("default");
    let ssid = matches.value_of("ssid").unwrap_or("resin-hotspot");
    let password = matches.value_of("password").unwrap_or("resin-hotspot");
    let timeout = matches.value_of("timeout").map_or(600, |v| v.parse::<i32>().unwrap());
    let verbose = matches.is_present("verbose");

    if verbose {
        println!("Interface: {}, SSID: {}, Password: {}, Timeout: {}",
                 interface,
                 ssid,
                 password,
                 timeout);
    }

    // Scan for access points

    // Start hotspot

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
