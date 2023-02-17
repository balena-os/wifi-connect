use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::process;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use network_manager::{
    AccessPoint, AccessPointCredentials, Connection, ConnectionState, Connectivity, Device,
    DeviceState, DeviceType, NetworkManager, Security, ServiceState,
};

use config::Config;
use dnsmasq::{start_dnsmasq, stop_dnsmasq};
use errors::*;
use exit::{exit, trap_exit_signals, ExitResult};
use server::start_server;

pub enum NetworkCommand {
    Activate,
    Timeout,
    Exit,
    Connect {
        ssid: String,
        identity: String,
        passphrase: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Network {
    ssid: String,
    security: String,
}

pub enum NetworkCommandResponse {
    Networks(Vec<Network>),
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
    fn new(config: &Config, exit_tx: &Sender<ExitResult>) -> Result<Self> {
        let (network_tx, network_rx) = channel();

        Self::spawn_trap_exit_signals(exit_tx, network_tx.clone());

        let manager = NetworkManager::new();
        debug!("NetworkManager connection initialized");

        let device = find_device(&manager, &config.interface)?;

        let access_points = get_access_points(&device)?;

        let portal_connection = Some(create_portal(&device, config)?);

        let dnsmasq = start_dnsmasq(config, &device)?;

        let (server_tx, server_rx) = channel();

        Self::spawn_server(config, exit_tx, server_rx, network_tx.clone());

        Self::spawn_activity_timeout(config, network_tx);

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
        let listening_port = config.listening_port;
        let exit_tx_server = exit_tx.clone();
        let ui_directory = config.ui_directory.clone();

        thread::spawn(move || {
            start_server(
                gateway,
                listening_port,
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
                    err.to_string()
                );
            }
        });
    }

    fn spawn_trap_exit_signals(exit_tx: &Sender<ExitResult>, network_tx: Sender<NetworkCommand>) {
        let exit_tx_trap = exit_tx.clone();

        thread::spawn(move || {
            if let Err(e) = trap_exit_signals() {
                exit(&exit_tx_trap, e);
                return;
            }

            if let Err(err) = network_tx.send(NetworkCommand::Exit) {
                error!("Sending NetworkCommand::Exit failed: {}", err.to_string());
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
                }
                NetworkCommand::Timeout => {
                    if !self.activated {
                        info!("Timeout reached. Exiting...");
                        return Ok(());
                    }
                }
                NetworkCommand::Exit => {
                    info!("Exiting...");
                    return Ok(());
                }
                NetworkCommand::Connect {
                    ssid,
                    identity,
                    passphrase,
                } => {
                    if self.connect(&ssid, &identity, &passphrase)? {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn receive_network_command(&self) -> Result<NetworkCommand> {
        match self.network_rx.recv() {
            Ok(command) => Ok(command),
            Err(e) => {
                // Sleep for a second, so that other threads may log error info.
                thread::sleep(Duration::from_secs(1));
                Err(e).chain_err(|| ErrorKind::RecvNetworkCommand)
            }
        }
    }

    fn stop(&mut self, exit_tx: &Sender<ExitResult>, result: ExitResult) {
        let _ = stop_dnsmasq(&mut self.dnsmasq);

        if let Some(ref connection) = self.portal_connection {
            let _ = stop_portal_impl(connection, &self.config);
        }

        let _ = exit_tx.send(result);
    }

    fn activate(&mut self) -> ExitResult {
        self.activated = true;

        let networks = get_networks(&self.access_points);

        self.server_tx
            .send(NetworkCommandResponse::Networks(networks))
            .chain_err(|| ErrorKind::SendAccessPointSSIDs)
    }

    fn connect(&mut self, ssid: &str, identity: &str, passphrase: &str) -> Result<bool> {
        delete_existing_connections_to_same_network(&self.manager, ssid);

        if let Some(ref connection) = self.portal_connection {
            stop_portal(connection, &self.config)?;
        }

        self.portal_connection = None;

        self.access_points = get_access_points(&self.device)?;

        if let Some(access_point) = find_access_point(&self.access_points, ssid) {
            let wifi_device = self.device.as_wifi_device().unwrap();

            info!("Connecting to access point '{}'...", ssid);

            let credentials = init_access_point_credentials(access_point, identity, passphrase);

            match wifi_device.connect(access_point, &credentials) {
                Ok((connection, state)) => {
                    if state == ConnectionState::Activated {
                        match wait_for_connectivity(&self.manager, 20) {
                            Ok(has_connectivity) => {
                                if has_connectivity {
                                    info!("Internet connectivity established");
                                } else {
                                    warn!("Cannot establish Internet connectivity");
                                }
                            }
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
                }
                Err(e) => {
                    warn!("Error connecting to access point '{}': {}", ssid, e);
                }
            }
        }

        self.access_points = get_access_points(&self.device)?;

        self.portal_connection = Some(create_portal(&self.device, &self.config)?);

        Ok(false)
    }
}

fn init_access_point_credentials(
    access_point: &AccessPoint,
    identity: &str,
    passphrase: &str,
) -> AccessPointCredentials {
    if access_point.security.contains(Security::ENTERPRISE) {
        AccessPointCredentials::Enterprise {
            identity: identity.to_string(),
            passphrase: passphrase.to_string(),
        }
    } else if access_point.security.contains(Security::WPA2)
        || access_point.security.contains(Security::WPA)
    {
        AccessPointCredentials::Wpa {
            passphrase: passphrase.to_string(),
        }
    } else if access_point.security.contains(Security::WEP) {
        AccessPointCredentials::Wep {
            passphrase: passphrase.to_string(),
        }
    } else {
        AccessPointCredentials::None
    }
}

pub fn process_network_commands(config: &Config, exit_tx: &Sender<ExitResult>) {
    let mut command_handler = match NetworkCommandHandler::new(config, exit_tx) {
        Ok(command_handler) => command_handler,
        Err(e) => {
            exit(exit_tx, e);
            return;
        }
    };

    command_handler.run(exit_tx);
}

pub fn init_networking(config: &Config) -> Result<()> {
    start_network_manager_service()?;

    delete_exising_wifi_connect_ap_profile(&config.ssid).chain_err(|| ErrorKind::DeleteAccessPoint)
}

pub fn find_device(manager: &NetworkManager, interface: &Option<String>) -> Result<Device> {
    if let Some(ref interface) = *interface {
        let device = manager
            .get_device_by_interface(interface)
            .chain_err(|| ErrorKind::DeviceByInterface(interface.clone()))?;

        info!("Targeted WiFi device: {}", interface);

        if *device.device_type() != DeviceType::WiFi {
            bail!(ErrorKind::NotAWiFiDevice(interface.clone()))
        }

        if device.get_state()? == DeviceState::Unmanaged {
            bail!(ErrorKind::UnmanagedDevice(interface.clone()))
        }

        Ok(device)
    } else {
        let devices = manager.get_devices()?;

        if let Some(device) = find_wifi_managed_device(devices)? {
            info!("WiFi device: {}", device.interface());
            Ok(device)
        } else {
            bail!(ErrorKind::NoWiFiDevice)
        }
    }
}

fn find_wifi_managed_device(devices: Vec<Device>) -> Result<Option<Device>> {
    for device in devices {
        if *device.device_type() == DeviceType::WiFi
            && device.get_state()? != DeviceState::Unmanaged
        {
            return Ok(Some(device));
        }
    }

    Ok(None)
}

fn get_access_points(device: &Device) -> Result<Vec<AccessPoint>> {
    get_access_points_impl(device).chain_err(|| ErrorKind::NoAccessPoints)
}

fn get_access_points_impl(device: &Device) -> Result<Vec<AccessPoint>> {
    let retries_allowed = 10;
    let mut retries = 0;

    // After stopping the hotspot we may have to wait a bit for the list
    // of access points to become available
    while retries < retries_allowed {
        let wifi_device = device.as_wifi_device().unwrap();
        let mut access_points = wifi_device.get_access_points()?;

        access_points.retain(|ap| ap.ssid().as_str().is_ok());

        // Purge access points with duplicate SSIDs
        let mut inserted = HashSet::new();
        access_points.retain(|ap| inserted.insert(ap.ssid.clone()));

        // Remove access points without SSID (hidden)
        access_points.retain(|ap| !ap.ssid().as_str().unwrap().is_empty());

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

fn get_networks(access_points: &[AccessPoint]) -> Vec<Network> {
    access_points.iter().map(get_network_info).collect()
}

fn get_network_info(access_point: &AccessPoint) -> Network {
    Network {
        ssid: access_point.ssid().as_str().unwrap().to_string(),
        security: get_network_security(access_point).to_string(),
    }
}

fn get_network_security(access_point: &AccessPoint) -> &str {
    if access_point.security.contains(Security::ENTERPRISE) {
        "enterprise"
    } else if access_point.security.contains(Security::WPA2)
        || access_point.security.contains(Security::WPA)
    {
        "wpa"
    } else if access_point.security.contains(Security::WEP) {
        "wep"
    } else {
        "none"
    }
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

fn create_portal(device: &Device, config: &Config) -> Result<Connection> {
    let portal_passphrase = config.passphrase.as_ref().map(|p| p as &str);

    create_portal_impl(device, &config.ssid, &config.gateway, &portal_passphrase)
        .chain_err(|| ErrorKind::CreateCaptivePortal)
}

fn create_portal_impl(
    device: &Device,
    ssid: &str,
    gateway: &Ipv4Addr,
    passphrase: &Option<&str>,
) -> Result<Connection> {
    info!("Starting access point...");
    let wifi_device = device.as_wifi_device().unwrap();
    let (portal_connection, _) = wifi_device.create_hotspot(ssid, *passphrase, Some(*gateway))?;
    info!("Access point '{}' created", ssid);
    Ok(portal_connection)
}

fn stop_portal(connection: &Connection, config: &Config) -> Result<()> {
    stop_portal_impl(connection, config).chain_err(|| ErrorKind::StopAccessPoint)
}

fn stop_portal_impl(connection: &Connection, config: &Config) -> Result<()> {
    info!("Stopping access point '{}'...", config.ssid);
    connection.deactivate()?;
    connection.delete()?;
    thread::sleep(Duration::from_secs(1));
    info!("Access point '{}' stopped", config.ssid);
    Ok(())
}

fn wait_for_connectivity(manager: &NetworkManager, timeout: u64) -> Result<bool> {
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

pub fn start_network_manager_service() -> Result<()> {
    let state = match NetworkManager::get_service_state() {
        Ok(state) => state,
        _ => {
            info!("Cannot get the NetworkManager service state");
            return Ok(());
        }
    };

    if state != ServiceState::Active {
        let state =
            NetworkManager::start_service(15).chain_err(|| ErrorKind::StartNetworkManager)?;
        if state != ServiceState::Active {
            bail!(ErrorKind::StartActiveNetworkManager);
        } else {
            info!("NetworkManager service started successfully");
        }
    } else {
        debug!("NetworkManager service already running");
    }

    Ok(())
}

fn delete_exising_wifi_connect_ap_profile(ssid: &str) -> Result<()> {
    let manager = NetworkManager::new();

    for connection in &manager.get_connections()? {
        if is_access_point_connection(connection) && is_same_ssid(connection, ssid) {
            info!(
                "Deleting already created by WiFi Connect access point connection profile: {:?}",
                connection.settings().ssid,
            );
            connection.delete()?;
        }
    }

    Ok(())
}

fn delete_existing_connections_to_same_network(manager: &NetworkManager, ssid: &str) {
    let connections = match manager.get_connections() {
        Ok(connections) => connections,
        Err(e) => {
            error!("Getting existing connections failed: {}", e);
            return;
        }
    };

    for connection in &connections {
        if is_wifi_connection(connection) && is_same_ssid(connection, ssid) {
            info!(
                "Deleting existing WiFi connection to the same network: {:?}",
                connection.settings().ssid,
            );

            if let Err(e) = connection.delete() {
                error!("Deleting existing WiFi connection failed: {}", e);
            }
        }
    }
}

fn is_same_ssid(connection: &Connection, ssid: &str) -> bool {
    connection_ssid_as_str(connection) == Some(ssid)
}

fn connection_ssid_as_str(connection: &Connection) -> Option<&str> {
    // An access point SSID could be random bytes and not a UTF-8 encoded string
    connection.settings().ssid.as_str().ok()
}

fn is_access_point_connection(connection: &Connection) -> bool {
    is_wifi_connection(connection) && connection.settings().mode == "ap"
}

fn is_wifi_connection(connection: &Connection) -> bool {
    connection.settings().kind == "802-11-wireless"
}
