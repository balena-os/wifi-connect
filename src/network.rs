use std::thread;
use std::process;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::error::Error;
use std::net::Ipv4Addr;

use network_manager::{AccessPoint, Connection, ConnectionState, Connectivity, Device, DeviceType,
                      NetworkManager, ServiceState};

use {exit, ExitResult};
use config::Config;
use dnsmasq::start_dnsmasq;
use server::start_server;

pub enum NetworkCommand {
    Activate,
    Timeout,
    Connect { ssid: String, passphrase: String },
}

pub enum NetworkCommandResponse {
    AccessPointsSsids(Vec<String>),
}

struct NetworkCommandHandler {
    manager: NetworkManager,
    device: Device,
    access_points: Vec<AccessPoint>,
    portal_connection: Option<Connection>,
    config: Config,
    dnsmasq: process::Child,
    server_tx: Sender<NetworkCommandResponse>,
    network_rx: Receiver<NetworkCommand>,
    activated: bool,
}

impl NetworkCommandHandler {
    fn new(config: &Config, exit_tx: &Sender<ExitResult>) -> Result<Self, String> {
        let manager = NetworkManager::new();
        debug!("NetworkManager connection initialized");

        let device = find_device(&manager, &config.interface)?;

        let access_points = get_access_points(&device)?;

        let portal_connection = Some(create_portal(&device, config)?);

        let dnsmasq = start_dnsmasq(config, &device)?;

        let (server_tx, server_rx) = channel();
        let (network_tx, network_rx) = channel();

        Self::spawn_server(config, exit_tx, server_rx, network_tx.clone());

        Self::spawn_activity_timeout(config, network_tx.clone());

        let config = config.clone();
        let activated = false;

        Ok(NetworkCommandHandler {
            manager,
            device,
            access_points,
            portal_connection,
            config,
            dnsmasq,
            server_tx,
            network_rx,
            activated,
        })
    }

    fn spawn_server(
        config: &Config,
        exit_tx: &Sender<ExitResult>,
        server_rx: Receiver<NetworkCommandResponse>,
        network_tx: Sender<NetworkCommand>,
    ) {
        let gateway = config.gateway;
        let exit_tx_server = exit_tx.clone();
        let ui_directory = config.ui_directory.clone();

        thread::spawn(move || {
            start_server(
                gateway,
                server_rx,
                network_tx,
                exit_tx_server,
                &ui_directory,
            );
        });
    }

    fn spawn_activity_timeout(config: &Config, network_tx: Sender<NetworkCommand>) {
        let activity_timeout = config.activity_timeout;

        if activity_timeout == 0 {
            return;
        }

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(activity_timeout));

            if let Err(err) = network_tx.send(NetworkCommand::Timeout) {
                error!(
                    "Sending NetworkCommand::Timeout failed: {}",
                    err.description()
                );
            }
        });
    }

    fn run(&mut self, exit_tx: &Sender<ExitResult>) {
        let result = self.run_loop();
        self.stop(exit_tx, result);
    }

    fn run_loop(&mut self) -> ExitResult {
        loop {
            let command = self.receive_network_command()?;

            match command {
                NetworkCommand::Activate => {
                    self.activate()?;
                },
                NetworkCommand::Timeout => {
                    if !self.activated {
                        info!("Timeout reached. Exiting...");
                        return Ok(());
                    }
                },
                NetworkCommand::Connect { ssid, passphrase } => {
                    if self.connect(&ssid, &passphrase)? {
                        return Ok(());
                    }
                },
            }
        }
    }

    fn receive_network_command(&self) -> Result<NetworkCommand, String> {
        match self.network_rx.recv() {
            Ok(command) => Ok(command),
            Err(e) => {
                // Sleep for a second, so that other threads may log error info.
                thread::sleep(Duration::from_secs(1));
                Err(format!(
                    "Receiving network command failed: {}",
                    e.description()
                ))
            },
        }
    }

    fn stop(&mut self, exit_tx: &Sender<ExitResult>, result: ExitResult) {
        let _ = self.dnsmasq.kill();

        if let Some(ref connection) = self.portal_connection {
            let _ = stop_portal_impl(connection, &self.config);
        }

        let _ = exit_tx.send(result);
    }

    fn activate(&mut self) -> ExitResult {
        self.activated = true;

        let access_points_ssids = get_access_points_ssids_owned(&self.access_points);

        if let Err(e) = self.server_tx
            .send(NetworkCommandResponse::AccessPointsSsids(
                access_points_ssids,
            )) {
            return Err(format!(
                "Sending access point ssids results failed: {}",
                e.description()
            ));
        }

        Ok(())
    }

    fn connect(&mut self, ssid: &str, passphrase: &str) -> Result<bool, String> {
        delete_connection_if_exists(&self.manager, ssid);

        if let Some(ref connection) = self.portal_connection {
            stop_portal(connection, &self.config)?;
        }

        self.portal_connection = None;

        self.access_points = get_access_points(&self.device)?;

        if let Some(access_point) = find_access_point(&self.access_points, ssid) {
            let wifi_device = self.device.as_wifi_device().unwrap();

            info!("Connecting to access point '{}'...", ssid);

            match wifi_device.connect(access_point, passphrase) {
                Ok((connection, state)) => {
                    if state == ConnectionState::Activated {
                        match wait_for_connectivity(&self.manager, 20) {
                            Ok(has_connectivity) => {
                                if has_connectivity {
                                    info!("Internet connectivity established");
                                } else {
                                    warn!("Cannot establish Internet connectivity");
                                }
                            },
                            Err(err) => error!("Getting Internet connectivity failed: {}", err),
                        }

                        return Ok(true);
                    }

                    if let Err(err) = connection.delete() {
                        error!("Deleting connection object failed: {}", err)
                    }

                    warn!(
                        "Connection to access point not activated '{}': {:?}",
                        ssid, state
                    );
                },
                Err(e) => {
                    warn!("Error connecting to access point '{}': {}", ssid, e);
                },
            }
        }

        self.access_points = get_access_points(&self.device)?;

        self.portal_connection = Some(create_portal(&self.device, &self.config)?);

        Ok(false)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(cyclomatic_complexity))]
pub fn process_network_commands(config: &Config, exit_tx: &Sender<ExitResult>) {
    let mut command_handler = match NetworkCommandHandler::new(config, exit_tx) {
        Ok(command_handler) => command_handler,
        Err(e) => {
            exit(exit_tx, e);
            return;
        },
    };

    command_handler.run(exit_tx);
}

pub fn init_networking() {
    start_network_manager_service();

    if let Err(err) = delete_access_point_profiles() {
        error!("Stopping access point failed: {}", err);
        process::exit(1);
    }
}

pub fn find_device(manager: &NetworkManager, interface: &Option<String>) -> Result<Device, String> {
    if let Some(ref interface) = *interface {
        let device = manager.get_device_by_interface(interface)?;

        if *device.device_type() == DeviceType::WiFi {
            info!("Targeted WiFi device: {}", interface);
            Ok(device)
        } else {
            Err(format!("Not a WiFi device: {}", interface))
        }
    } else {
        let devices = manager.get_devices()?;

        let index = devices
            .iter()
            .position(|d| *d.device_type() == DeviceType::WiFi);

        if let Some(index) = index {
            info!("WiFi device: {}", devices[index].interface());
            Ok(devices[index].clone())
        } else {
            Err("Cannot find a WiFi device".to_string())
        }
    }
}

fn get_access_points(device: &Device) -> Result<Vec<AccessPoint>, String> {
    get_access_points_impl(device).map_err(|e| format!("Getting access points failed: {}", e))
}

fn get_access_points_impl(device: &Device) -> Result<Vec<AccessPoint>, String> {
    let retries_allowed = 10;
    let mut retries = 0;

    // After stopping the hotspot we may have to wait a bit for the list
    // of access points to become available
    while retries < retries_allowed {
        let wifi_device = device.as_wifi_device().unwrap();
        let mut access_points = wifi_device.get_access_points()?;

        access_points.retain(|ap| ap.ssid().as_str().is_ok());

        if !access_points.is_empty() {
            info!(
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

fn find_access_point<'a>(access_points: &'a [AccessPoint], ssid: &str) -> Option<&'a AccessPoint> {
    for access_point in access_points.iter() {
        if let Ok(access_point_ssid) = access_point.ssid().as_str() {
            if access_point_ssid == ssid {
                return Some(access_point);
            }
        }
    }

    None
}

fn create_portal(device: &Device, config: &Config) -> Result<Connection, String> {
    let portal_passphrase = config.passphrase.as_ref().map(|p| p as &str);

    create_portal_impl(device, &config.ssid, &config.gateway, &portal_passphrase)
        .map_err(|e| format!("Creating the captive portal failed: {}", e))
}

fn create_portal_impl(
    device: &Device,
    ssid: &str,
    gateway: &Ipv4Addr,
    passphrase: &Option<&str>,
) -> Result<Connection, String> {
    info!("Starting access point...");
    let wifi_device = device.as_wifi_device().unwrap();
    let (portal_connection, _) = wifi_device.create_hotspot(ssid, *passphrase, Some(*gateway))?;
    info!("Access point '{}' created", ssid);
    Ok(portal_connection)
}

fn stop_portal(connection: &Connection, config: &Config) -> Result<(), String> {
    stop_portal_impl(connection, config)
        .map_err(|e| format!("Stopping the access point failed: {}", e))
}

fn stop_portal_impl(connection: &Connection, config: &Config) -> Result<(), String> {
    info!("Stopping access point '{}'...", config.ssid);
    connection.deactivate()?;
    connection.delete()?;
    thread::sleep(Duration::from_secs(1));
    info!("Access point '{}' stopped", config.ssid);
    Ok(())
}

fn wait_for_connectivity(manager: &NetworkManager, timeout: u64) -> Result<bool, String> {
    let mut total_time = 0;

    loop {
        let connectivity = manager.get_connectivity()?;

        if connectivity == Connectivity::Full || connectivity == Connectivity::Limited {
            debug!(
                "Connectivity established: {:?} / {}s elapsed",
                connectivity, total_time
            );

            return Ok(true);
        } else if total_time >= timeout {
            debug!(
                "Timeout reached in waiting for connectivity: {:?} / {}s elapsed",
                connectivity, total_time
            );

            return Ok(false);
        }

        ::std::thread::sleep(::std::time::Duration::from_secs(1));

        total_time += 1;

        debug!(
            "Still waiting for connectivity: {:?} / {}s elapsed",
            connectivity, total_time
        );
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
                        error!(
                            "Starting the NetworkManager service state failed: {:?}",
                            err
                        );
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

fn delete_access_point_profiles() -> Result<(), String> {
    let manager = NetworkManager::new();

    let connections = manager.get_connections()?;

    for connection in connections {
        if &connection.settings().kind == "802-11-wireless" && &connection.settings().mode == "ap" {
            debug!(
                "Deleting access point connection profile: {:?}",
                connection.settings().ssid,
            );
            connection.delete()?;
        }
    }

    Ok(())
}

fn delete_connection_if_exists(manager: &NetworkManager, ssid: &str) {
    let connections = match manager.get_connections() {
        Ok(connections) => connections,
        Err(e) => {
            error!("Getting existing connections failed: {}", e);
            return;
        },
    };

    for connection in connections {
        if let Ok(connection_ssid) = connection.settings().ssid.as_str() {
            if &connection.settings().kind == "802-11-wireless" && connection_ssid == ssid {
                info!(
                    "Deleting existing WiFi connection: {:?}",
                    connection.settings().ssid,
                );

                if let Err(e) = connection.delete() {
                    error!("Deleting existing WiFi connection failed: {}", e);
                }
            }
        }
    }
}
