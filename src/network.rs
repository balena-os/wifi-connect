use std::thread;
use std::process;
use std::time::Duration;
use std::sync::mpsc::{Sender, channel};
use std::error::Error;
use std::net::Ipv4Addr;

use network_manager::{NetworkManager, Device, DeviceState, DeviceType, Connection, AccessPoint,
                      ConnectionState, ServiceState, Connectivity};

use {exit, ExitResult};
use config::Config;
use dnsmasq::start_dnsmasq;
use server::start_server;

pub enum NetworkCommand {
    Activate,
    Connect { ssid: String, password: String },
}

pub enum NetworkCommandResponse {
    AccessPointsSsids(Vec<String>),
}

#[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
pub fn process_network_commands(config: &Config, exit_tx: &Sender<ExitResult>) {
    let manager = NetworkManager::with_method_timeout(config.timeout);
    debug!("Network Manager connection initialized");

    let device = match find_device(&manager, &config.interface) {
        Ok(device) => device,
        Err(e) => {
            return exit(exit_tx, e);
        },
    };

    let mut access_points = match get_access_points(&device) {
        Ok(access_points) => access_points,
        Err(e) => {
            return exit(exit_tx, format!("Getting access points failed: {}", e));
        },
    };

    let hotspot_ssid = &config.ssid;
    let hotspot_password = config.password.as_ref().map(|p| p as &str);

    let mut hotspot_connection =
        match create_hotspot(&device, &config.ssid, &config.gateway, &hotspot_password) {
            Ok(connection) => Some(connection),
            Err(e) => {
                return exit(exit_tx, format!("Creating the hotspot failed: {}", e));
            },
        };

    let dnsmasq = start_dnsmasq(config, &device).unwrap();

    let (server_tx, server_rx) = channel();
    let (network_tx, network_rx) = channel();

    let exit_tx_server = exit_tx.clone();
    let gateway = config.gateway;
    let ui_path = config.ui_path.clone();
    thread::spawn(move || {
        start_server(gateway, server_rx, network_tx, exit_tx_server, &ui_path);
    });

    loop {
        let command = match network_rx.recv() {
            Ok(command) => command,
            Err(e) => {
                // Sleep for a second, so that other threads may log error info.
                thread::sleep(Duration::from_secs(1));
                return exit_with_error(
                    exit_tx,
                    dnsmasq,
                    hotspot_connection,
                    hotspot_ssid,
                    format!("Receiving network command failed: {}", e.description()),
                );
            },
        };

        match command {
            NetworkCommand::Activate => {
                let access_points_ssids = get_access_points_ssids_owned(&access_points);

                if let Err(e) = server_tx.send(NetworkCommandResponse::AccessPointsSsids(
                    access_points_ssids,
                ))
                {
                    return exit_with_error(
                        exit_tx,
                        dnsmasq,
                        hotspot_connection,
                        hotspot_ssid,
                        format!(
                            "Sending access point ssids results failed: {}",
                            e.description()
                        ),
                    );
                }
            },
            NetworkCommand::Connect {
                ssid,
                password,
            } => {
                if let Some(connection) = hotspot_connection {
                    let result = stop_hotspot(&connection, &config.ssid);
                    if let Err(e) = result {
                        return exit_with_error(
                            exit_tx,
                            dnsmasq,
                            Some(connection),
                            hotspot_ssid,
                            format!("Stopping the hotspot failed: {}", e),
                        );
                    }
                    hotspot_connection = None;
                }

                access_points = match get_access_points(&device) {
                    Ok(access_points) => access_points,
                    Err(e) => {
                        return exit_with_error(
                            exit_tx,
                            dnsmasq,
                            hotspot_connection,
                            hotspot_ssid,
                            format!("Getting access points failed: {}", e),
                        );
                    },
                };

                {
                    let (access_point, access_point_ssid) =
                        find_access_point(&access_points, &ssid).unwrap();

                    let wifi_device = device.as_wifi_device().unwrap();

                    info!("Connecting to access point '{}'...", access_point_ssid);

                    match wifi_device.connect(access_point, &password as &str) {
                        Ok((connection, state)) => {
                            if state == ConnectionState::Activated {
                                match wait_for_connectivity(&manager, config.timeout) {
                                    Ok(has_connectivity) => {
                                        if has_connectivity {
                                            info!("Connectivity established");

                                            return exit_ok(
                                                exit_tx,
                                                dnsmasq,
                                                hotspot_connection,
                                                hotspot_ssid,
                                            );
                                        } else {
                                            warn!("Cannot establish connectivity");
                                        }
                                    },
                                    Err(err) => error!("Getting connectivity failed: {}", err),
                                }
                            }

                            if let Err(err) = connection.delete() {
                                error!("Deleting connection object failed: {}", err)
                            }

                            warn!(
                                "Connection to access point not activated '{}': {:?}",
                                access_point_ssid,
                                state
                            );
                        },
                        Err(e) => {
                            warn!(
                                "Error connecting to access point '{}': {}",
                                access_point_ssid,
                                e
                            );
                        },
                    }
                }

                access_points = match get_access_points(&device) {
                    Ok(access_points) => access_points,
                    Err(e) => {
                        return exit_with_error(
                            exit_tx,
                            dnsmasq,
                            hotspot_connection,
                            hotspot_ssid,
                            format!("Getting access points failed: {}", e),
                        );
                    },
                };

                hotspot_connection = match create_hotspot(
                    &device,
                    &config.ssid,
                    &config.gateway,
                    &hotspot_password,
                ) {
                    Ok(connection) => Some(connection),
                    Err(e) => {
                        return exit_with_error(
                            exit_tx,
                            dnsmasq,
                            hotspot_connection,
                            hotspot_ssid,
                            format!("Creating the hotspot failed: {}", e),
                        );
                    },
                };
            },
        }
    }
}

pub fn handle_existing_wifi_connections(clear: bool, interface: &Option<String>) {
    let manager = NetworkManager::new();

    if let Err(err) = stop_access_point(&manager) {
        error!("Stopping access point failed: {}", err);
        process::exit(1);
    }

    let connections = get_existing_wifi_connections(&manager);

    if clear {
        if let Err(err) = clear_wifi_connections(connections) {
            error!("Clearing Wi-Fi connections failed: {}", err);
            process::exit(1);
        }
    } else if !connections.is_empty() {
        if has_full_connectivity(&manager) && has_activated_device(&manager, interface) {
            info!("The device has a network connection");
            process::exit(0);
        }

        try_activate_wifi_connection(connections);
    }
}

fn has_full_connectivity(manager: &NetworkManager) -> bool {
    match manager.get_connectivity() {
        Ok(connectivity) => {
            if connectivity == Connectivity::Full {
                info!("The host appears to be able to reach the full Internet");
                true
            } else {
                info!("The host does not appear to be able to reach the full Internet");
                false
            }
        },
        Err(err) => {
            error!("Assuming no connectivity as checking for network connectivity failed: {}", err);
            false
        },
    }
}

fn has_activated_device(manager: &NetworkManager, interface: &Option<String>) -> bool {
    let device = match find_device(manager, interface) {
        Ok(device) => device,
        Err(e) => {
            error!("{}", e);
            return false;
        },
    };

    match device.get_state() {
        Ok(state) => state == DeviceState::Activated,
        Err(e) => {
            error!("Cannot get device state: {}", e);
            false
        },
    }
}

fn clear_wifi_connections(connections: Vec<Connection>) -> Result<(), String> {
    for connection in connections {
        if &connection.settings().kind == "802-11-wireless" {
            info!(
                "Deleting Wi-Fi connection profile to {:?}...",
                connection.settings().ssid,
            );

            debug!("ID [{}] UUID {}", connection.settings().id, connection.settings().uuid);
            connection.delete()?;
        }
    }

    Ok(())
}

fn get_existing_wifi_connections(manager: &NetworkManager) -> Vec<Connection> {
    let mut connections = match manager.get_connections() {
        Ok(connections) => connections,
        Err(err) => {
            error!("Getting existing connections failed: {}", err);
            process::exit(1)
        },
    };

    connections.retain(|c| &c.settings().kind == "802-11-wireless" && &c.settings().mode != "ap");

    connections
}

fn try_activate_wifi_connection(connections: Vec<Connection>) {
    for connection in connections {
        match connection.activate() {
            Ok(state) => {
                if state == ConnectionState::Activated {
                    info!(
                        "Activated Wi-Fi connection to {:?}: [{}] {}",
                        connection.settings().ssid,
                        connection.settings().id,
                        connection.settings().uuid
                    );
                    process::exit(0);
                } else {
                    debug!(
                        "Cannot activate Wi-Fi connection to {:?}: [{}] {}",
                        connection.settings().ssid,
                        connection.settings().id,
                        connection.settings().uuid
                    );
                }
            },
            Err(err) => {
                warn!("Activating existing Wi-Fi connection failed: {}", err);
            },
        }
    }

    info!("Cannot activate any existing Wi-Fi connection");
    process::exit(0);
}

pub fn find_device(manager: &NetworkManager, interface: &Option<String>) -> Result<Device, String> {
    if let Some(ref interface) = *interface {
        let device = manager.get_device_by_interface(interface)?;

        if *device.device_type() == DeviceType::WiFi {
            info!("Targeted Wi-Fi device: {}", interface);
            Ok(device)
        } else {
            Err(format!("Not a Wi-Fi device: {}", interface))
        }
    } else {
        let devices = manager.get_devices()?;

        let index = devices.iter().position(
            |d| *d.device_type() == DeviceType::WiFi,
        );

        if let Some(index) = index {
            info!("Wi-Fi device: {}", devices[index].interface());
            Ok(devices[index].clone())
        } else {
            Err("Cannot find a Wi-Fi device".to_string())
        }
    }
}

fn get_access_points(device: &Device) -> Result<Vec<AccessPoint>, String> {
    let retries_allowed = 10;
    let mut retries = 0;

    // After stopping the hotspot we may have to wait a bit for the list
    // of access points to become available
    while retries < retries_allowed {
        let wifi_device = device.as_wifi_device().unwrap();
        let mut access_points = wifi_device.get_access_points()?;

        access_points.retain(|ap| ap.ssid().as_str().is_ok());

        if !access_points.is_empty() {
            info!("Access points: {:?}", get_access_points_ssids(&access_points));
            return Ok(access_points);
        }

        retries += 1;
        debug!("No access points found - retry #{}", retries);
        thread::sleep(Duration::from_secs(1));
    }

    warn!("No access points found - giving up...");
    Ok(vec![])
}

fn get_access_points_ssids(access_points: &[AccessPoint]) -> Vec<&str> {
    access_points
        .iter()
        .map(|ap| ap.ssid().as_str().unwrap())
        .collect()
}

fn get_access_points_ssids_owned(access_points: &[AccessPoint]) -> Vec<String> {
    access_points
        .iter()
        .map(|ap| ap.ssid().as_str().unwrap().to_string())
        .collect()
}

fn find_access_point<'a>(
    access_points: &'a [AccessPoint],
    ssid: &str,
) -> Option<(&'a AccessPoint, &'a str)> {
    for access_point in access_points.iter() {
        if let Ok(access_point_ssid) = access_point.ssid().as_str() {
            if access_point_ssid == ssid {
                return Some((access_point, access_point_ssid));
            }
        }
    }

    None
}

fn create_hotspot(
    device: &Device,
    ssid: &str,
    gateway: &Ipv4Addr,
    password: &Option<&str>,
) -> Result<Connection, String> {
    info!("Creating hotspot...");
    let wifi_device = device.as_wifi_device().unwrap();
    let (hotspot_connection, _) = wifi_device.create_hotspot(ssid, *password, Some(*gateway))?;
    info!("Hotspot '{}' created", ssid);
    Ok(hotspot_connection)
}

fn stop_hotspot(connection: &Connection, ssid: &str) -> Result<(), String> {
    info!("Stopping hotspot '{}'...", ssid);
    connection.deactivate()?;
    connection.delete()?;
    thread::sleep(Duration::from_secs(1));
    info!("Hotspot '{}' stopped", ssid);
    Ok(())
}

fn exit_with_error(
    exit_tx: &Sender<ExitResult>,
    dnsmasq: process::Child,
    connection: Option<Connection>,
    ssid: &str,
    error: String,
) {
    exit_with_result(exit_tx, dnsmasq, connection, ssid, Err(error));
}

fn exit_ok(
    exit_tx: &Sender<ExitResult>,
    dnsmasq: process::Child,
    connection: Option<Connection>,
    ssid: &str,
) {
    exit_with_result(exit_tx, dnsmasq, connection, ssid, Ok(()));
}

fn exit_with_result(
    exit_tx: &Sender<ExitResult>,
    mut dnsmasq: process::Child,
    connection: Option<Connection>,
    ssid: &str,
    result: ExitResult,
) {
    let _ = dnsmasq.kill();

    if let Some(connection) = connection {
        let _ = stop_hotspot(&connection, ssid);
    }

    let _ = exit_tx.send(result);
}

fn wait_for_connectivity(manager: &NetworkManager, timeout: u64) -> Result<bool, String> {
    let mut total_time = 0;

    loop {
        let connectivity = manager.get_connectivity()?;

        if connectivity == Connectivity::Full || connectivity == Connectivity::Limited {
            debug!("Connectivity established: {:?} / {}s elapsed", connectivity, total_time);

            return Ok(true);
        } else if total_time >= timeout {
            debug!(
                "Timeout reached in waiting for connectivity: {:?} / {}s elapsed",
                connectivity,
                total_time
            );

            return Ok(false);
        }

        ::std::thread::sleep(::std::time::Duration::from_secs(1));

        total_time += 1;

        debug!("Still waiting for connectivity: {:?} / {}s elapsed", connectivity, total_time);
    }
}

pub fn start_network_manager_service() {
    match NetworkManager::get_service_state() {
        Ok(state) => {
            if state != ServiceState::Active {
                match NetworkManager::start_service(15) {
                    Ok(state) => {
                        if state != ServiceState::Active {
                            error!(
                                "Cannot start the NetworkManager service with active state: {:?}",
                                state
                            );
                            process::exit(1);
                        } else {
                            info!("NetworkManager service started successfully");
                        }
                    },
                    Err(err) => {
                        error!("Starting the NetworkManager service state failed: {:?}", err);
                        process::exit(1);
                    },
                }
            } else {
                debug!("NetworkManager service already running");
            }
        },
        Err(err) => {
            error!("Getting the NetworkManager service state failed: {:?}", err);
            process::exit(1);
        },
    }
}

fn stop_access_point(manager: &NetworkManager) -> Result<(), String> {
    let connections = manager.get_active_connections()?;

    for connection in connections {
        if &connection.settings().kind == "802-11-wireless" && &connection.settings().mode == "ap" {
            debug!(
                "Deleting active access point connection profile to {:?}",
                connection.settings().ssid,
            );
            connection.delete()?;
        }
    }

    Ok(())
}
