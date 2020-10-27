use clap::{App, Arg};

use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::path::PathBuf;
use std::ffi::OsStr;

const DEFAULT_GATEWAY: &str = "192.168.42.1";
const DEFAULT_DHCP_RANGE: &str = "192.168.42.2,192.168.42.254";
const DEFAULT_SSID: &str = "WiFi Connect";
const DEFAULT_ACTIVITY_TIMEOUT: &str = "0";
const DEFAULT_UI_DIRECTORY: &str = "ui";
const DEFAULT_LISTENING_PORT: &str = "80";

#[derive(Clone)]
pub struct Config {
    pub interface: Option<String>,
    pub ssid: String,
    pub passphrase: Option<String>,
    pub gateway: Ipv4Addr,
    pub dhcp_range: String,
    pub listening_port: u16,
    pub activity_timeout: u64,
    pub ui_directory: PathBuf,
}

pub fn get_config() -> Config {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("portal-interface")
                .short("i")
                .long("portal-interface")
                .value_name("interface")
                .help("Wireless network interface to be used by WiFi Connect")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("portal-ssid")
                .short("s")
                .long("portal-ssid")
                .value_name("ssid")
                .help(&format!(
                    "SSID of the captive portal WiFi network (default: {})",
                    DEFAULT_SSID
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("portal-passphrase")
                .short("p")
                .long("portal-passphrase")
                .value_name("passphrase")
                .help("WPA2 Passphrase of the captive portal WiFi network (default: none)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("portal-gateway")
                .short("g")
                .long("portal-gateway")
                .value_name("gateway")
                .help(&format!(
                    "Gateway of the captive portal WiFi network (default: {})",
                    DEFAULT_GATEWAY
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("portal-dhcp-range")
                .short("d")
                .long("portal-dhcp-range")
                .value_name("dhcp_range")
                .help(&format!(
                    "DHCP range of the WiFi network (default: {})",
                    DEFAULT_DHCP_RANGE
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("portal-listening-port")
                .short("o")
                .long("portal-listening-port")
                .value_name("listening_port")
                .help(&format!(
                    "Listening port of the captive portal web server (default: {})",
                    DEFAULT_LISTENING_PORT
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("activity-timeout")
                .short("a")
                .long("activity-timeout")
                .value_name("activity_timeout")
                .help("Exit if no activity for the specified time (seconds) (default: none)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ui-directory")
                .short("u")
                .long("ui-directory")
                .value_name("ui_directory")
                .help(&format!(
                    "Web UI directory location (default: {})",
                    DEFAULT_UI_DIRECTORY
                ))
                .takes_value(true),
        )
        .get_matches();

    let interface: Option<String> = matches.value_of("portal-interface").map_or_else(
        || env::var("PORTAL_INTERFACE").ok(),
        |v| Some(v.to_string()),
    );

    let ssid: String = matches.value_of("portal-ssid").map_or_else(
        || env::var("PORTAL_SSID").unwrap_or_else(|_| DEFAULT_SSID.to_string()),
        String::from,
    );

    let passphrase: Option<String> = matches.value_of("portal-passphrase").map_or_else(
        || env::var("PORTAL_PASSPHRASE").ok(),
        |v| Some(v.to_string()),
    );

    let gateway = Ipv4Addr::from_str(&matches.value_of("portal-gateway").map_or_else(
        || env::var("PORTAL_GATEWAY").unwrap_or_else(|_| DEFAULT_GATEWAY.to_string()),
        String::from,
    )).expect("Cannot parse gateway address");

    let dhcp_range = matches.value_of("portal-dhcp-range").map_or_else(
        || env::var("PORTAL_DHCP_RANGE").unwrap_or_else(|_| DEFAULT_DHCP_RANGE.to_string()),
        String::from,
    );

    let listening_port = matches
        .value_of("portal-listening-port")
        .map_or_else(
            || {
                env::var("PORTAL_LISTENING_PORT")
                    .unwrap_or_else(|_| DEFAULT_LISTENING_PORT.to_string())
            },
            String::from,
        )
        .parse::<u16>()
        .expect("Cannot parse listening port number");

    let activity_timeout = u64::from_str(&matches.value_of("activity-timeout").map_or_else(
        || env::var("ACTIVITY_TIMEOUT").unwrap_or_else(|_| DEFAULT_ACTIVITY_TIMEOUT.to_string()),
        String::from,
    )).expect("Cannot parse activity timeout");

    let ui_directory = get_ui_directory(matches.value_of("ui-directory"));

    Config {
        interface: interface,
        ssid: ssid,
        passphrase: passphrase,
        gateway: gateway,
        dhcp_range: dhcp_range,
        listening_port: listening_port,
        activity_timeout: activity_timeout,
        ui_directory: ui_directory,
    }
}

fn get_ui_directory(cmd_ui_directory: Option<&str>) -> PathBuf {
    if let Some(ui_directory) = cmd_ui_directory {
        return PathBuf::from(ui_directory);
    }

    if let Ok(ui_directory) = env::var("UI_DIRECTORY") {
        return PathBuf::from(ui_directory);
    }

    if let Some(install_ui_directory) = get_install_ui_directory() {
        return install_ui_directory;
    }

    PathBuf::from(DEFAULT_UI_DIRECTORY)
}

/// Checks whether `WiFi Connect` is running from install path and whether the
/// UI directory is present in a corresponding location
/// e.g. /usr/local/sbin/wifi-connect -> /usr/local/share/wifi-connect/ui
fn get_install_ui_directory() -> Option<PathBuf> {
    if let Ok(exe_path) = env::current_exe() {
        if let Ok(mut path) = exe_path.canonicalize() {
            path.pop();

            match path.file_name() {
                Some(file_name) => {
                    if file_name != OsStr::new("sbin") {
                        // not executing from `sbin` folder
                        return None;
                    }
                },
                None => return None,
            }

            path.pop();
            path.push("share");
            path.push(env!("CARGO_PKG_NAME"));
            path.push("ui");

            if path.is_dir() {
                return Some(path);
            }
        }
    }

    None
}
