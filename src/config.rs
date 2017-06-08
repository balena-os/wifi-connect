use clap::{Arg, App};

use std::env;

pub struct Config {
    pub interface: Option<String>,
    pub ssid: String,
    pub password: Option<String>,
    pub clear: bool,
}

pub fn get_config() -> Config {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("interface")
                .short("i")
                .long("interface")
                .value_name("INTERFACE")
                .help("Hotspot interface")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("ssid")
                .short("s")
                .long("ssid")
                .value_name("SSID")
                .help("Hotspot SSID")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .value_name("PASSWORD")
                .help("Hotspot password ")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("clear")
                .short("c")
                .long("clear")
                .value_name("CLEAR")
                .help("Clear saved Wi-Fi credentials")
                .takes_value(true)
        )
        .get_matches();

    let interface: Option<String> =
        matches
            .value_of("interface")
            .map_or_else(|| env::var("PORTAL_INTERFACE").ok(), |v| Some(v.to_string()));

    let ssid: String = matches
        .value_of("ssid")
        .map_or_else(
            || env::var("PORTAL_SSID").unwrap_or_else(|_| "ResinAP".to_string()),
            String::from,
        );

    let password: Option<String> =
        matches
            .value_of("password")
            .map_or_else(|| env::var("PORTAL_PASSPHRASE").ok(), |v| Some(v.to_string()));

    let clear = matches.value_of("clear").map_or(true, |v| !(v == "false"));

    Config {
        interface: interface,
        ssid: ssid,
        password: password,
        clear: clear,
    }
}
