use clap::{Arg, App};

pub struct CliOptions {
    pub interface: Option<String>,
    pub ssid: String,
    pub password: Option<String>,
    pub timeout: u64,
}

pub fn parse_cli_options() -> CliOptions {
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

    CliOptions {
        interface: interface,
        ssid: ssid,
        password: password,
        timeout: timeout,
    }
}
