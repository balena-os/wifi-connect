use std::thread;
use std::process;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};
use std::error::Error;
use std::net::Ipv4Addr;

use network_manager::{NetworkManager, Device, DeviceType, Connection, AccessPoint, ConnectionState};

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

pub fn process_network_commands(
    config: &Config,
    network_tx: Sender<NetworkCommand>,
    network_rx: Receiver<NetworkCommand>,
    server_tx: Sender<NetworkCommandResponse>,
    server_rx: Receiver<NetworkCommandResponse>,
    exit_tx: Sender<ExitResult>,
) {
    let manager = NetworkManager::new();
    debug!("Network Manager connection initialized");

    let device = match find_device(&manager, &config.interface) {
        Ok(device) => device,
        Err(e) => {
            return exit(&exit_tx, e);
        },
    };

    let mut activated = false;

    let mut access_points = match get_access_points(&device) {
        Ok(access_points) => access_points,
        Err(e) => {
            return exit(&exit_tx, format!("Getting access points failed: {}", e));
        },
    };

    let hotspot_password = config.password.as_ref().map(|p| p as &str);

    let mut hotspot_connection = match create_hotspot(&device, &config.ssid, &config.gateway, &hotspot_password) {
        Ok(connection) => Some(connection),
        Err(e) => {
            return exit(&exit_tx, format!("Creating the hotspot failed: {}", e));
        },
    };

    let dnsmasq = start_dnsmasq(&config, &device).unwrap();

    let exit_tx_server = exit_tx.clone();
    let gateway = config.gateway.clone();
    thread::spawn(move || { start_server(gateway, server_rx, network_tx, exit_tx_server); });

    'main_loop: loop {
        let command = match network_rx.recv() {
            Ok(command) => command,
            Err(e) => {
                return exit_with_error(
                    &exit_tx,
                    dnsmasq,
                    format!("Receiving network command failed: {}", e.description()),
                );
            },
        };

        match command {
            NetworkCommand::Activate => {
                // Access points are retrieved and hotspot is started before
                // the first command arrives.
                if activated {
                    if hotspot_connection.is_some() {
                        let result = stop_hotspot(&hotspot_connection.unwrap(), &config.ssid);
                        if let Err(e) = result {
                            return exit_with_error(
                                &exit_tx,
                                dnsmasq,
                                format!("Stopping the hotspot failed: {}", e),
                            );
                        }
                    }

                    access_points = match get_access_points(&device) {
                        Ok(access_points) => access_points,
                        Err(e) => {
                            return exit_with_error(
                                &exit_tx,
                                dnsmasq,
                                format!("Getting access points failed: {}", e),
                            );
                        },
                    };

                    hotspot_connection =
                        match create_hotspot(&device, &config.ssid, &config.gateway, &hotspot_password) {
                            Ok(connection) => Some(connection),
                            Err(e) => {
                                return exit_with_error(
                                    &exit_tx,
                                    dnsmasq,
                                    format!("Creating the hotspot failed: {}", e),
                                );
                            },
                        };
                };

                let access_points_ssids = get_access_points_ssids_owned(&access_points);

                activated = true;

                if let Err(e) = server_tx.send(NetworkCommandResponse::AccessPointsSsids(
                    access_points_ssids,
                ))
                {
                    return exit_with_error(
                        &exit_tx,
                        dnsmasq,
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
                if hotspot_connection.is_some() {
                    if let Err(e) = stop_hotspot(&hotspot_connection.unwrap(), &config.ssid) {
                        return exit_with_error(&exit_tx, dnsmasq, format!("Stopping the hotspot failed: {}", e));
                    }
                    hotspot_connection = None;
                }

                let access_points = match get_access_points(&device) {
                    Ok(access_points) => access_points,
                    Err(e) => {
                        return exit_with_error(
                            &exit_tx,
                            dnsmasq,
                            format!("Getting access points failed: {}", e),
                        );
                    },
                };

                for access_point in access_points {
                    if let Ok(access_point_ssid) = access_point.ssid().as_str() {
                        if access_point_ssid == ssid {
                            let wifi_device = device.as_wifi_device().unwrap();

                            debug!("Connecting to access point '{}'...", access_point_ssid);

                            match wifi_device.connect(&access_point, &password as &str) {
                                Ok((connection, state)) => {
                                    if state == ConnectionState::Activated {
                                        let _ = exit_tx.send(Ok(()));

                                        return;
                                    } else {
                                        if let Err(err) = connection.delete() {
                                            error!("Deleting connection object failed: {}", err)
                                        }

                                        warn!(
                                            "Connection to access point not activated '{}': {:?}",
                                            access_point_ssid,
                                            state
                                        );

                                        continue 'main_loop;
                                    }
                                },
                                Err(e) => {
                                    warn!(
                                        "Error connecting to access point '{}': {}",
                                        access_point_ssid,
                                        e
                                    );

                                    continue 'main_loop;
                                },
                            }
                        }
                    }
                }
            },
        }
    }
}

pub fn handle_existing_wifi_connections(clear: bool) {
    let manager = NetworkManager::new();

    if clear {
        if let Err(err) = clear_wifi_connections(&manager) {
            error!("Clearing Wi-Fi connections failed: {}", err);
            process::exit(1);
        }
    } else {
        match find_and_activate_wifi_connection(&manager) {
            Err(err) => {
                error!("Finding and activating Wi-Fi connection failed: {}", err);
                process::exit(1);
            },
            Ok(activated) => {
                match activated {
                    ConnectionActivated::Yes(connection) => {
                        match connection.settings().ssid.as_str() {
                            Ok(ssid) => {
                                info!("Existing Wi-Fi connection found and activated: {}", ssid)
                            },
                            Err(_) => {
                                info!(
                                    "Existing Wi-Fi connection found and activated: {:?}",
                                    connection.settings().ssid.as_bytes()
                                )
                            },
                        }
                        process::exit(0);
                    },
                    ConnectionActivated::No => {
                        info!("Cannot find and activate an existing Wi-Fi connection");
                    },
                }
            },
        }
    }
}

fn clear_wifi_connections(manager: &NetworkManager) -> Result<(), String> {
    let connections = manager.get_connections()?;

    for connection in connections {
        if &connection.settings().kind == "802-11-wireless" {
            debug!(
                "Deleting Wi-Fi connection profile to {:?}: [{}] {}",
                connection.settings().ssid,
                connection.settings().id,
                connection.settings().uuid
            );
            connection.delete()?;
        }
    }

    Ok(())
}

enum ConnectionActivated {
    Yes(Connection),
    No,
}

fn find_and_activate_wifi_connection(
    manager: &NetworkManager,
) -> Result<ConnectionActivated, String> {
    let connections = manager.get_connections()?;

    for connection in connections {
        if &connection.settings().kind == "802-11-wireless" {
            let state = connection.activate()?;

            if state == ConnectionState::Activated {
                debug!(
                    "Activated Wi-Fi connection to {:?}: [{}] {}",
                    connection.settings().ssid,
                    connection.settings().id,
                    connection.settings().uuid
                );
                return Ok(ConnectionActivated::Yes(connection));
            } else {
                debug!(
                    "Cannot activate Wi-Fi connection to {:?}: [{}] {}",
                    connection.settings().ssid,
                    connection.settings().id,
                    connection.settings().uuid
                );
            }
        }
    }

    debug!("No connection activated");
    Ok(ConnectionActivated::No)
}

fn find_device(manager: &NetworkManager, interface: &Option<String>) -> Result<Device, String> {
    if let Some(ref interface) = *interface {
        let device = manager.get_device_by_interface(interface)?;

        if *device.device_type() == DeviceType::WiFi {
            info!("Targeted Wi-Fi device found: {}", interface);
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
            info!("Wi-Fi device found: {}", devices[index].interface());
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
            debug!("Access points: {:?}", get_access_points_ssids(&access_points));
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

fn create_hotspot(
    device: &Device,
    ssid: &str,
    gateway: &Ipv4Addr,
    password: &Option<&str>,
) -> Result<Connection, String> {
    info!("Creating hotspot...");
    let wifi_device = device.as_wifi_device().unwrap();
    let (hotspot_connection, _) = wifi_device.create_hotspot(ssid, *password, Some(gateway.clone()))?;
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

pub fn exit_with_error(exit_tx: &Sender<ExitResult>, mut dnsmasq: process::Child, error: String) {
    dnsmasq.kill().unwrap();

    let _ = exit_tx.send(Err(error));
}
