use std::cell::RefCell;
use std::collections::HashSet;
use std::process;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use ascii::AsciiStr;

use glib::translate::FromGlib;

use futures_channel::oneshot;
use futures_core::future::Future;

use nm::{ActiveConnectionExt, Cast, ConnectionExt, DeviceExt, SettingIPConfigExt};

use tokio::runtime::Runtime;

use crate::config::{Config, DEFAULT_GATEWAY};
use crate::dnsmasq::{start_dnsmasq, stop_dnsmasq};
use crate::errors::*;
use crate::exit::{exit, trap_exit_signals, ExitResult};
use crate::server::start_server;
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Network {
    ssid: String,
    security: String,
}

pub enum NetworkCommandResponse {
    Networks(Vec<Network>),
}

struct NetworkCommandHandler {
    client: nm::Client,
    device: nm::Device,
    networks: Vec<Network>,
    portal_connection: Option<nm::ActiveConnection>,
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

        let client = nm::Client::new(nm::NONE_CANCELLABLE).unwrap();
        debug!("NetworkManager connection initialized");

        let device = find_device(&client, &config.interface)?;

        println!("Device: {:?}", device);

        let access_points = get_access_points(&device)?;
        let networks = get_networks(&access_points);

        let portal_connection = Some(create_portal(&client, &device, config)?);

        let dnsmasq = start_dnsmasq(config, &device)?;

        let (server_tx, server_rx) = channel();

        Self::spawn_server(config, exit_tx, server_rx, network_tx.clone());

        Self::spawn_activity_timeout(config, network_tx);

        let config = config.clone();
        let activated = false;

        Ok(NetworkCommandHandler {
            client,
            device,
            networks,
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
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                start_server(
                    gateway,
                    listening_port,
                    server_rx,
                    network_tx,
                    exit_tx_server,
                    &ui_directory,
                )
                .await
            });
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
                error!("Sending NetworkCommand::Timeout failed: {}", err);
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
                error!("Sending NetworkCommand::Exit failed: {}", err);
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

        if let Some(ref active_connection) = self.portal_connection {
            let _ = stop_portal(&self.client, active_connection, &self.config);
        }

        let _ = exit_tx.send(result);
    }

    fn activate(&mut self) -> ExitResult {
        self.activated = true;

        self.server_tx
            .send(NetworkCommandResponse::Networks(self.networks.clone()))
            .chain_err(|| ErrorKind::SendAccessPointSSIDs)
    }

    fn connect(&mut self, ssid: &str, identity: &str, passphrase: &str) -> Result<bool> {
        let context = glib::MainContext::default();

        context.block_on(delete_exising_wifi_connect_ap_profile(
            Some(&self.client),
            ssid,
        ))?;

        if let Some(ref active_connection) = self.portal_connection {
            stop_portal(&self.client, active_connection, &self.config)?;
        }

        self.portal_connection = None;

        let access_points = get_access_points(&self.device)?;

        if let Some(access_point) = find_access_point(&access_points, ssid) {
            info!("Connecting to access point '{}'...", ssid);

            let credentials = init_access_point_credentials(access_point, identity, passphrase);

            match context.block_on(connect_to_access_point(
                &self.client,
                &self.device,
                access_point,
                &credentials,
            )) {
                Ok(active_connection) => {
                    if active_connection.state() == nm::ActiveConnectionState::Activated {
                        match wait_for_connectivity(&self.client, 20) {
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

                    if let Err(err) = context.block_on(
                        active_connection
                            .connection()
                            .unwrap()
                            .delete_async_future(),
                    ) {
                        error!("Deleting connection object failed: {}", err)
                    }

                    warn!(
                        "Connection to access point not activated '{}': {:?}",
                        ssid,
                        active_connection.state()
                    );
                }
                Err(e) => {
                    warn!("Error connecting to access point '{}': {}", ssid, e);
                }
            }
        }

        let access_points = get_access_points(&self.device)?;
        self.networks = get_networks(&access_points);

        self.portal_connection = Some(create_portal(&self.client, &self.device, &self.config)?);

        Ok(false)
    }
}

#[derive(Debug)]
pub enum AccessPointCredentials {
    None,
    Wep {
        passphrase: String,
    },
    Wpa {
        passphrase: String,
    },
    Enterprise {
        identity: String,
        passphrase: String,
    },
}

fn init_access_point_credentials(
    access_point: &nm::AccessPoint,
    identity: &str,
    passphrase: &str,
) -> AccessPointCredentials {
    let security = get_access_point_security(access_point);
    if security.contains(Security::ENTERPRISE) {
        AccessPointCredentials::Enterprise {
            identity: identity.to_string(),
            passphrase: passphrase.to_string(),
        }
    } else if security.contains(Security::WPA2) || security.contains(Security::WPA) {
        AccessPointCredentials::Wpa {
            passphrase: passphrase.to_string(),
        }
    } else if security.contains(Security::WEP) {
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
    let context = glib::MainContext::default();

    context
        .block_on(delete_exising_wifi_connect_ap_profile(None, &config.ssid))
        .chain_err(|| ErrorKind::DeleteAccessPoint)
}

pub fn find_device(client: &nm::Client, interface: &Option<String>) -> Result<nm::Device> {
    if let Some(ref interface) = *interface {
        get_exact_device(client, interface)
    } else {
        find_any_wifi_device(client)
    }
}

fn get_exact_device(client: &nm::Client, interface: &str) -> Result<nm::Device> {
    let device = client
        .device_by_iface(interface)
        .chain_err(|| ErrorKind::DeviceByInterface(interface.to_string()))?;

    if device.device_type() != nm::DeviceType::Wifi {
        bail!(ErrorKind::NotAWiFiDevice(interface.to_string()))
    }

    if device.state() == nm::DeviceState::Unmanaged {
        bail!(ErrorKind::UnmanagedDevice(interface.to_string()))
    }

    Ok(device)
}

fn find_any_wifi_device(client: &nm::Client) -> Result<nm::Device> {
    for device in client.devices() {
        if device.device_type() == nm::DeviceType::Wifi
            && device.state() != nm::DeviceState::Unmanaged
        {
            return Ok(device);
        }
    }

    bail!(ErrorKind::NoWiFiDevice)
}

fn get_access_points(device: &nm::Device) -> Result<Vec<nm::AccessPoint>> {
    get_access_points_impl(device).chain_err(|| ErrorKind::NoAccessPoints)
}

fn get_access_points_impl(device: &nm::Device) -> Result<Vec<nm::AccessPoint>> {
    let retries_allowed = 10;
    let mut retries = 0;

    // After stopping the hotspot we may have to wait a bit for the list
    // of access points to become available
    while retries < retries_allowed {
        let wifi_device = device.downcast_ref::<nm::DeviceWifi>().unwrap();
        let mut access_points = wifi_device.access_points();

        access_points.retain(|ap| ssid_to_string(ap.ssid()).is_some());

        // Purge access points with duplicate SSIDs
        let mut inserted = HashSet::new();
        access_points.retain(|ap| inserted.insert(ssid_to_string(ap.ssid()).unwrap()));

        // Remove access points without SSID (hidden)
        access_points.retain(|ap| !ssid_to_string(ap.ssid()).unwrap().is_empty());

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

fn get_access_points_ssids(access_points: &[nm::AccessPoint]) -> Vec<String> {
    access_points
        .iter()
        .map(|ap| ssid_to_string(ap.ssid()).unwrap())
        .collect()
}

fn get_networks(access_points: &[nm::AccessPoint]) -> Vec<Network> {
    access_points
        .iter()
        .map(|ap| get_network_info(ap))
        .collect()
}

fn get_network_info(access_point: &nm::AccessPoint) -> Network {
    Network {
        ssid: ssid_to_string(access_point.ssid()).unwrap(),
        security: get_network_security(access_point).to_string(),
    }
}

fn get_network_security(access_point: &nm::AccessPoint) -> &str {
    let security = get_access_point_security(access_point);
    if security.contains(Security::ENTERPRISE) {
        "enterprise"
    } else if security.contains(Security::WPA2) || security.contains(Security::WPA) {
        "wpa"
    } else if security.contains(Security::WEP) {
        "wep"
    } else {
        "none"
    }
}

bitflags! {
    pub struct Security: u32 {
        const NONE         = 0b0000_0000;
        const WEP          = 0b0000_0001;
        const WPA          = 0b0000_0010;
        const WPA2         = 0b0000_0100;
        const ENTERPRISE   = 0b0000_1000;
    }
}

fn get_access_point_security(access_point: &nm::AccessPoint) -> Security {
    let flags = access_point.flags();

    let wpa_flags = access_point.wpa_flags();

    let rsn_flags = access_point.rsn_flags();

    let mut security = Security::NONE;

    if flags.contains(nm::_80211ApFlags::PRIVACY)
        && wpa_flags == nm::_80211ApSecurityFlags::NONE
        && rsn_flags == nm::_80211ApSecurityFlags::NONE
    {
        security |= Security::WEP;
    }

    if wpa_flags != nm::_80211ApSecurityFlags::NONE {
        security |= Security::WPA;
    }

    if rsn_flags != nm::_80211ApSecurityFlags::NONE {
        security |= Security::WPA2;
    }

    if wpa_flags.contains(nm::_80211ApSecurityFlags::KEY_MGMT_802_1X)
        || rsn_flags.contains(nm::_80211ApSecurityFlags::KEY_MGMT_802_1X)
    {
        security |= Security::ENTERPRISE;
    }

    security
}

fn find_access_point<'a>(
    access_points: &'a [nm::AccessPoint],
    ssid: &str,
) -> Option<&'a nm::AccessPoint> {
    for access_point in access_points.iter() {
        if let Some(access_point_ssid) = ssid_to_string(access_point.ssid()) {
            if access_point_ssid == ssid {
                return Some(access_point);
            }
        }
    }

    None
}

fn create_portal(
    client: &nm::Client,
    device: &nm::Device,
    config: &Config,
) -> Result<nm::ActiveConnection> {
    let context = glib::MainContext::default();

    let password = config.passphrase.as_ref().map(|p| p as &str);

    context
        .block_on(create_portal_impl(
            client,
            device,
            &config.ssid,
            DEFAULT_GATEWAY,
            &password,
        ))
        .chain_err(|| ErrorKind::CreateCaptivePortal)
}

async fn create_portal_impl(
    client: &nm::Client,
    device: &nm::Device,
    ssid: &str,
    gateway: &str,
    passphrase: &Option<&str>,
) -> Result<nm::ActiveConnection> {
    let interface = device.iface().unwrap();
    let connection = create_ap_connection(interface.as_str(), ssid, gateway, passphrase)?;

    let active_connection = client
        .add_and_activate_connection_async_future(Some(&connection), device, None)
        .await?;
    //        .context("Failed to add and activate connection")?;

    let (sender, receiver) = oneshot::channel::<Result<()>>();
    let sender = Rc::new(RefCell::new(Some(sender)));

    active_connection.connect_state_changed(move |active_connection, state, _| {
        let sender = sender.clone();
        let active_connection = active_connection.clone();
        spawn_local(async move {
            let state = unsafe { nm::ActiveConnectionState::from_glib(state as _) };
            println!("Active connection state: {:?}", state);

            let exit = match state {
                nm::ActiveConnectionState::Activated => {
                    println!("Successfully activated");
                    Some(Ok(()))
                }
                nm::ActiveConnectionState::Deactivated => {
                    println!("Connection deactivated");
                    if let Some(remote_connection) = active_connection.connection() {
                        Some(
                            remote_connection
                                .delete_async_future()
                                .await
                                .chain_err(|| ErrorKind::CreateCaptivePortal),
                        )
                        //.context("Failed to delete connection"),
                    } else {
                        Some(Err(
                            "Failed to get remote connection from active connection".into(),
                        ))
                    }
                }
                _ => None,
            };
            if let Some(result) = exit {
                let sender = sender.borrow_mut().take();
                if let Some(sender) = sender {
                    sender.send(result).expect("Sender dropped");
                }
            }
        });
    });

    if let Err(err) = receiver.await? {
        Err(err)
    } else {
        Ok(active_connection)
    }
}

pub fn spawn_local<F: Future<Output = ()> + 'static>(f: F) {
    glib::MainContext::ref_thread_default().spawn_local(f);
}

fn create_ap_connection(
    interface: &str,
    ssid: &str,
    address: &str,
    passphrase: &Option<&str>,
) -> Result<nm::SimpleConnection> {
    let connection = nm::SimpleConnection::new();

    let s_connection = nm::SettingConnection::new();
    s_connection.set_type(Some(&nm::SETTING_WIRELESS_SETTING_NAME));
    s_connection.set_id(Some(ssid));
    s_connection.set_autoconnect(false);
    s_connection.set_interface_name(Some(interface));
    connection.add_setting(&s_connection);

    let s_wireless = nm::SettingWireless::new();
    s_wireless.set_ssid(Some(&(ssid.as_bytes().into())));
    s_wireless.set_band(Some("bg"));
    s_wireless.set_hidden(false);
    s_wireless.set_mode(Some(&nm::SETTING_WIRELESS_MODE_AP));
    connection.add_setting(&s_wireless);

    if let Some(password) = passphrase {
        let s_wireless_security = nm::SettingWirelessSecurity::new();
        s_wireless_security.set_key_mgmt(Some("wpa-psk"));
        s_wireless_security.set_psk(Some(password));
        connection.add_setting(&s_wireless_security);
    }

    let s_ip4 = nm::SettingIP4Config::new();
    let address = nm::IPAddress::new(libc::AF_INET, address, 24).unwrap(); //context("Failed to parse address")?;
    s_ip4.add_address(&address);
    s_ip4.set_method(Some(&nm::SETTING_IP4_CONFIG_METHOD_MANUAL));
    connection.add_setting(&s_ip4);

    Ok(connection)
}

fn stop_portal(
    client: &nm::Client,
    active_connection: &nm::ActiveConnection,
    config: &Config,
) -> Result<()> {
    let context = glib::MainContext::default();

    context
        .block_on(stop_portal_impl(client, active_connection, config))
        .chain_err(|| ErrorKind::StopAccessPoint)
}

async fn stop_portal_impl(
    client: &nm::Client,
    active_connection: &nm::ActiveConnection,
    config: &Config,
) -> Result<()> {
    info!("Stopping access point '{}'...", config.ssid);
    client
        .deactivate_connection_async_future(active_connection)
        .await?;
    active_connection
        .connection()
        .unwrap()
        .delete_async_future()
        .await?;
    thread::sleep(Duration::from_secs(1));
    info!("Access point '{}' stopped", config.ssid);
    Ok(())
}

pub async fn connect_to_access_point(
    client: &nm::Client,
    device: &nm::Device,
    access_point: &nm::AccessPoint,
    credentials: &AccessPointCredentials,
) -> Result<nm::ActiveConnection> {
    let connection =
        create_station_connection(&ssid_to_string(access_point.ssid()).unwrap(), credentials)?;

    let active_connection = client
        .add_and_activate_connection_async_future(Some(&connection), device, None)
        .await?;
    //        .context("Failed to add and activate connection")?;

    let (sender, receiver) = oneshot::channel::<Result<()>>();
    let sender = Rc::new(RefCell::new(Some(sender)));

    active_connection.connect_state_changed(move |active_connection, state, _| {
        let sender = sender.clone();
        let active_connection = active_connection.clone();
        spawn_local(async move {
            let state = unsafe { nm::ActiveConnectionState::from_glib(state as _) };
            println!("Active connection state: {:?}", state);

            let exit = match state {
                nm::ActiveConnectionState::Activated => {
                    println!("Successfully activated");
                    Some(Ok(()))
                }
                nm::ActiveConnectionState::Deactivated => {
                    println!("Connection deactivated");
                    if let Some(remote_connection) = active_connection.connection() {
                        Some(
                            remote_connection
                                .delete_async_future()
                                .await
                                .chain_err(|| ErrorKind::CreateCaptivePortal),
                        )
                        //.context("Failed to delete connection"),
                    } else {
                        Some(Err(
                            "Failed to get remote connection from active connection".into(),
                        ))
                    }
                }
                _ => None,
            };
            if let Some(result) = exit {
                let sender = sender.borrow_mut().take();
                if let Some(sender) = sender {
                    sender.send(result).expect("Sender dropped");
                }
            }
        });
    });

    if let Err(err) = receiver.await? {
        Err(err)
    } else {
        Ok(active_connection)
    }
}

fn create_station_connection(
    ssid: &str,
    credentials: &AccessPointCredentials,
) -> Result<nm::SimpleConnection> {
    let connection = nm::SimpleConnection::new();

    let s_connection = nm::SettingConnection::new();
    s_connection.set_type(Some(&nm::SETTING_WIRELESS_SETTING_NAME));
    connection.add_setting(&s_connection);

    let s_wireless = nm::SettingWireless::new();
    s_wireless.set_ssid(Some(&(ssid.as_bytes().into())));
    connection.add_setting(&s_wireless);

    match *credentials {
        AccessPointCredentials::Wep { ref passphrase } => {
            let s_wireless_security = nm::SettingWirelessSecurity::new();
            s_wireless_security.set_wep_key_type(nm::WepKeyType::Passphrase);
            s_wireless_security.set_wep_key0(Some(verify_ascii_password(passphrase)?));
            connection.add_setting(&s_wireless_security);
        }
        AccessPointCredentials::Wpa { ref passphrase } => {
            let s_wireless_security = nm::SettingWirelessSecurity::new();
            s_wireless_security.set_key_mgmt(Some("wpa-psk"));
            s_wireless_security.set_psk(Some(verify_ascii_password(passphrase)?));
            connection.add_setting(&s_wireless_security);
        }
        AccessPointCredentials::Enterprise {
            ref identity,
            ref passphrase,
        } => {
            let s_wireless_security = nm::SettingWirelessSecurity::new();
            s_wireless_security.set_key_mgmt(Some("wpa-eap"));
            connection.add_setting(&s_wireless_security);

            let s_enterprise = nm::Setting8021x::new();
            s_enterprise.set_eap(&["peap"]);
            s_enterprise.set_identity(Some(identity as &str));
            s_enterprise.set_password(Some(passphrase as &str));
            s_enterprise.set_phase2_auth(Some("mschapv2"));
            connection.add_setting(&s_enterprise);
        }
        AccessPointCredentials::None => {}
    };

    Ok(connection)
}

fn verify_ascii_password(password: &str) -> Result<&str> {
    match AsciiStr::from_ascii(password) {
        Err(e) => Err(e).chain_err(|| ErrorKind::PreSharedKey("Not an ASCII password".into())),
        Ok(p) => {
            if p.len() < 8 {
                bail!(ErrorKind::PreSharedKey(format!(
                    "Password length should be at least 8 characters: {} len",
                    p.len()
                )))
            } else if p.len() > 64 {
                bail!(ErrorKind::PreSharedKey(format!(
                    "Password length should not exceed 64: {} len",
                    p.len()
                )))
            } else {
                Ok(password)
            }
        }
    }
}

fn wait_for_connectivity(client: &nm::Client, timeout: u64) -> Result<bool> {
    let mut total_time = 0;

    loop {
        let connectivity = client.connectivity();

        if connectivity == nm::ConnectivityState::Full
            || connectivity == nm::ConnectivityState::Limited
        {
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

async fn delete_exising_wifi_connect_ap_profile(
    client: Option<&nm::Client>,
    ssid: &str,
) -> Result<()> {
    let connections = if let Some(client) = client {
        client.connections()
    } else {
        let client = nm::Client::new(nm::NONE_CANCELLABLE).unwrap();
        client.connections()
    };

    for connection in connections {
        let c = connection.clone().upcast::<nm::Connection>();
        if is_access_point_connection(&c) && is_same_ssid(&c, ssid) {
            info!(
                "Deleting already created by WiFi Connect access point connection profile: {:?}",
                ssid,
            );
            connection.delete_async_future().await?;
        }
    }

    Ok(())
}

fn is_same_ssid(connection: &nm::Connection, ssid: &str) -> bool {
    connection_ssid_as_str(&connection) == Some(ssid.to_string())
}

fn connection_ssid_as_str(connection: &nm::Connection) -> Option<String> {
    ssid_to_string(connection.setting_wireless()?.ssid())
}

fn ssid_to_string(ssid: Option<glib::Bytes>) -> Option<String> {
    // An access point SSID could be random bytes and not a UTF-8 encoded string
    std::str::from_utf8(&ssid?).ok().map(str::to_owned)
}

fn is_access_point_connection(connection: &nm::Connection) -> bool {
    is_wifi_connection(&connection) && is_access_point_mode(&connection)
}

fn is_access_point_mode(connection: &nm::Connection) -> bool {
    if let Some(setting) = connection.setting_wireless() {
        if let Some(mode) = setting.mode() {
            return mode == *nm::SETTING_WIRELESS_MODE_AP;
        }
    }

    false
}

fn is_wifi_connection(connection: &nm::Connection) -> bool {
    if let Some(setting) = connection.setting_connection() {
        if let Some(connection_type) = setting.connection_type() {
            return connection_type == *nm::SETTING_WIRELESS_SETTING_NAME;
        }
    }

    false
}
