use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};
use std::error::Error;

use network_manager::{NetworkManager, Device, DeviceType, Connection, AccessPoint};

use cli::CliOptions;

pub enum NetworkCommand {
    Activate,
    Connect { ssid: String, password: String },
}

pub fn process_network_commands(
    cli_options: CliOptions,
    network_rx: Receiver<NetworkCommand>,
    server_tx: Sender<Vec<String>>,
    shutdown_tx: Sender<Option<String>>,
) {
    let manager = NetworkManager::new();

    debug!("Network Manager connection initialized");

    let device = match find_device(&manager, &cli_options.interface) {
        Ok(device) => device,
        Err(e) => {
            shutdown(&shutdown_tx, e);
            return;
        }
    };

    let mut activated = false;

    let mut access_points = match get_access_points(&device) {
        Ok(access_points) => access_points,
        Err(e) => {
            shutdown(&shutdown_tx, format!("Getting access points failed: {}", e));
            return;
        }
    };

    let hotspot_password = cli_options.password.as_ref().map(|p| p as &str);

    let mut hotspot_connection =
        match create_hotspot(&device, &cli_options.ssid, &hotspot_password) {
            Ok(connection) => Some(connection),
            Err(e) => {
                shutdown(&shutdown_tx, format!("Creating the hotspot failed: {}", e));
                return;
            }
        };

    'main_loop: loop {
        let command = match network_rx.recv() {
            Ok(command) => command,
            Err(e) => {
                shutdown(
                    &shutdown_tx,
                    format!("Receiving network command failed: {}", e.description()),
                );
                return;
            }
        };

        match command {
            NetworkCommand::Activate => {
                // First time the access points are retrieved and hotspot is started
                // before the first command arrives
                if activated {
                    if hotspot_connection.is_some() {
                        if let Err(e) = stop_hotspot(
                            &hotspot_connection.unwrap(),
                            &cli_options.ssid,
                        ) {
                            shutdown(&shutdown_tx, format!("Stopping the hotspot failed: {}", e));
                            return;
                        }
                    }

                    access_points = match get_access_points(&device) {
                        Ok(access_points) => access_points,
                        Err(e) => {
                            shutdown(&shutdown_tx, format!("Getting access points failed: {}", e));
                            return;
                        }
                    };

                    hotspot_connection = match create_hotspot(
                        &device,
                        &cli_options.ssid,
                        &hotspot_password,
                    ) {
                        Ok(connection) => Some(connection),
                        Err(e) => {
                            shutdown(&shutdown_tx, format!("Creating the hotspot failed: {}", e));
                            return;
                        }
                    };
                };

                let access_points_ssids = get_access_points_ssids_owned(&access_points);

                activated = true;

                if let Err(e) = server_tx.send(access_points_ssids) {
                    shutdown(
                        &shutdown_tx,
                        format!(
                            "Sending access point ssids results failed: {}",
                            e.description()
                        ),
                    );
                    return;
                }
            }
            NetworkCommand::Connect { ssid, password } => {
                if hotspot_connection.is_some() {
                    if let Err(e) = stop_hotspot(&hotspot_connection.unwrap(), &cli_options.ssid) {
                        shutdown(&shutdown_tx, format!("Stopping the hotspot failed: {}", e));
                        return;
                    }
                    hotspot_connection = None;
                }

                let access_points = match get_access_points(&device) {
                    Ok(access_points) => access_points,
                    Err(e) => {
                        shutdown(&shutdown_tx, format!("Getting access points failed: {}", e));
                        return;
                    }
                };

                for access_point in access_points {
                    if let Ok(access_point_ssid) = access_point.ssid().as_str() {
                        if access_point_ssid == &ssid {
                            let wifi_device = device.as_wifi_device().unwrap();

                            debug!("Connecting to access point '{}'...", access_point_ssid);

                            match wifi_device.connect(&access_point, &password as &str) {
                                Ok(_) => {
                                    let _ = shutdown_tx.send(None);

                                    return;
                                }
                                Err(e) => {
                                    warn!(
                                        "Cannot connect to access point '{}': {}",
                                        access_point_ssid,
                                        e
                                    );

                                    continue 'main_loop;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn shutdown(shutdown_tx: &Sender<Option<String>>, error: String) {
    let _ = shutdown_tx.send(Some(error));
}

fn find_device(manager: &NetworkManager, interface: &Option<String>) -> Result<Device, String> {
    if let &Some(ref interface) = interface {
        let device = manager.get_device_by_interface(interface)?;

        if *device.device_type() == DeviceType::WiFi {
            info!("Targeted Wi-Fi device found: {}", interface);
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

        if access_points.len() != 0 {
            debug!(
                "Access points: {:?}",
                get_access_points_ssids(&access_points)
            );
            return Ok(access_points);
        }

        retries += 1;
        debug!("No access points found - retry #{}", retries);
        thread::sleep(Duration::from_secs(1));
    }

    warn!("No access points found - giving up...");
    return Ok(vec![]);
}

fn get_access_points_ssids<'a>(access_points: &'a Vec<AccessPoint>) -> Vec<&'a str> {
    access_points
        .iter()
        .map(|ap| ap.ssid().as_str().unwrap())
        .collect()
}

fn get_access_points_ssids_owned(access_points: &Vec<AccessPoint>) -> Vec<String> {
    access_points
        .iter()
        .map(|ap| ap.ssid().as_str().unwrap().to_string())
        .collect()
}

fn create_hotspot(
    device: &Device,
    ssid: &str,
    password: &Option<&str>,
) -> Result<Connection, String> {
    info!("Creating hotspot...");
    let wifi_device = device.as_wifi_device().unwrap();
    let (hotspot_connection, _) = wifi_device.create_hotspot(&ssid as &str, *password)?;
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
