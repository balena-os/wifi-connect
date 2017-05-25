use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Sender, Receiver};

use network_manager::{NetworkManager, Device, DeviceType, Connection, AccessPoint};

use cli::CliOptions;

pub enum NetworkCommand {
    Activate,
    Connect { ssid: String, password: String },
}

pub fn process_network_commands(cli_options: CliOptions,
                                network_rx: Receiver<NetworkCommand>,
                                server_tx: Sender<Vec<String>>,
                                shutdown_tx: Sender<()>) {
    let manager = NetworkManager::new();
    let device = find_device(&manager, &cli_options.interface).unwrap();

    let mut access_points_option = Some(get_access_points(&device).unwrap());

    let hotspot_password = cli_options.password.as_ref().map(|p| p as &str);
    let mut hotspot_connection = None;

    loop {
        let command = network_rx.recv().unwrap();

        match command {
            NetworkCommand::Activate => {
                if let Some(ref connection) = hotspot_connection {
                    stop_hotspot(connection).unwrap();
                }

                // First time the access points are retrieved before the command arrives
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
                    Some(create_hotspot(&device, &cli_options.ssid, &hotspot_password).unwrap());

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

fn find_device(manager: &NetworkManager, interface: &Option<String>) -> Result<Device, String> {
    if let &Some(ref interface) = interface {
        let device = manager.get_device_by_interface(interface)?;

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
