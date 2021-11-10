use anyhow::{anyhow, bail, Context, Result};

use tokio::sync::oneshot;

use glib::{MainContext, MainLoop};

use std::collections::HashSet;
use std::future::Future;

use serde::Serialize;

use nm::*;

const WIFI_SCAN_TIMEOUT_SECONDS: usize = 45;

type TokioResponder = oneshot::Sender<Result<NetworkResponse>>;

#[derive(Debug)]
pub enum NetworkCommand {
    CheckConnectivity,
    ListConnections,
    ListWiFiNetworks,
}

pub struct NetworkRequest {
    responder: TokioResponder,
    command: NetworkCommand,
}

impl NetworkRequest {
    pub fn new(responder: TokioResponder, command: NetworkCommand) -> Self {
        NetworkRequest { responder, command }
    }
}

pub enum NetworkResponse {
    CheckConnectivity(Connectivity),
    ListConnections(ConnectionList),
    ListWiFiNetworks(NetworkList),
}

#[derive(Serialize)]
pub struct Connectivity {
    pub connectivity: String,
}

impl Connectivity {
    fn new(connectivity: String) -> Self {
        Connectivity { connectivity }
    }
}

#[derive(Serialize)]
pub struct ConnectionList {
    pub connections: Vec<ConnectionDetails>,
}

impl ConnectionList {
    fn new(connections: Vec<ConnectionDetails>) -> Self {
        ConnectionList { connections }
    }
}

#[derive(Serialize)]
pub struct ConnectionDetails {
    pub id: String,
    pub uuid: String,
}

impl ConnectionDetails {
    fn new(id: String, uuid: String) -> Self {
        ConnectionDetails { id, uuid }
    }
}

#[derive(Serialize)]
pub struct NetworkList {
    pub networks: Vec<NetworkDetails>,
}

impl NetworkList {
    fn new(networks: Vec<NetworkDetails>) -> Self {
        NetworkList { networks }
    }
}

#[derive(Serialize)]
pub struct NetworkDetails {
    pub ssid: String,
    pub strength: u8,
}

impl NetworkDetails {
    fn new(ssid: String, strength: u8) -> Self {
        NetworkDetails { ssid, strength }
    }
}

pub fn create_channel() -> (glib::Sender<NetworkRequest>, glib::Receiver<NetworkRequest>) {
    MainContext::channel(glib::PRIORITY_DEFAULT)
}

pub fn run_network_manager_loop(glib_receiver: glib::Receiver<NetworkRequest>) {
    let context = MainContext::new();
    let loop_ = MainLoop::new(Some(&context), false);

    context
        .with_thread_default(|| {
            glib_receiver.attach(None, dispatch_command_requests);

            context.spawn_local(init_network());

            loop_.run();
        })
        .unwrap();
}

async fn init_network() {
    delete_exising_wifi_connect_ap_profile(crate::config::DEFAULT_SSID)
        .await
        .unwrap();
}

fn dispatch_command_requests(command_request: NetworkRequest) -> glib::Continue {
    let NetworkRequest { responder, command } = command_request;
    match command {
        NetworkCommand::CheckConnectivity => spawn(check_connectivity(), responder),
        NetworkCommand::ListConnections => spawn(list_connections(), responder),
        NetworkCommand::ListWiFiNetworks => spawn(list_wifi_networks(), responder),
    };
    glib::Continue(true)
}

fn spawn(
    command_future: impl Future<Output = Result<NetworkResponse>> + 'static,
    responder: TokioResponder,
) {
    let context = MainContext::ref_thread_default();
    context.spawn_local(execute_and_respond(command_future, responder));
}

async fn execute_and_respond(
    command_future: impl Future<Output = Result<NetworkResponse>> + 'static,
    responder: TokioResponder,
) {
    let result = command_future.await;
    let _ = responder.send(result);
}

async fn check_connectivity() -> Result<NetworkResponse> {
    let client = create_client().await?;

    let connectivity = client
        .check_connectivity_async_future()
        .await
        .context("Failed to execute check connectivity")?;

    Ok(NetworkResponse::CheckConnectivity(Connectivity::new(
        connectivity.to_string(),
    )))
}

async fn list_connections() -> Result<NetworkResponse> {
    let client = create_client().await?;

    let all_connections: Vec<_> = client
        .connections()
        .into_iter()
        .map(|c| c.upcast::<Connection>())
        .collect();

    let mut connections = Vec::new();

    for connection in all_connections {
        if let Some(setting_connection) = connection.setting_connection() {
            if let Some(id) = setting_connection.id() {
                if let Some(uuid) = setting_connection.uuid() {
                    connections.push(ConnectionDetails::new(id.to_string(), uuid.to_string()));
                }
            }
        }
    }

    Ok(NetworkResponse::ListConnections(ConnectionList::new(
        connections,
    )))
}

async fn list_wifi_networks() -> Result<NetworkResponse> {
    let client = create_client().await?;

    let device = find_any_wifi_device(&client)?;

    scan_wifi(&device).await?;

    let access_points = get_nearby_access_points(&device);

    let networks = access_points
        .iter()
        .map(|ap| NetworkDetails::new(ssid_to_string(ap.ssid()).unwrap(), ap.strength()))
        .collect::<Vec<_>>();

    Ok(NetworkResponse::ListWiFiNetworks(NetworkList::new(
        networks,
    )))
}

async fn scan_wifi(device: &DeviceWifi) -> Result<()> {
    let prescan = utils_get_timestamp_msec();

    device
        .request_scan_async_future()
        .await
        .context("Failed to request WiFi scan")?;

    for _ in 0..WIFI_SCAN_TIMEOUT_SECONDS {
        if prescan < device.last_scan() {
            break;
        }

        glib::timeout_future_seconds(1).await;
    }

    Ok(())
}

fn get_nearby_access_points(device: &DeviceWifi) -> Vec<AccessPoint> {
    let mut access_points = device.access_points();

    // Purge non-string SSIDs
    access_points.retain(|ap| ssid_to_string(ap.ssid()).is_some());

    // Purge access points with duplicate SSIDs
    let mut inserted = HashSet::new();
    access_points.retain(|ap| inserted.insert(ssid_to_string(ap.ssid()).unwrap()));

    // Purge access points without SSID (hidden)
    access_points.retain(|ap| !ssid_to_string(ap.ssid()).unwrap().is_empty());

    // Sort access points by signal strength first and then ssid
    access_points.sort_by_key(|ap| (ap.strength(), ssid_to_string(ap.ssid()).unwrap()));
    access_points.reverse();

    access_points
}

fn ssid_to_string(ssid: Option<glib::Bytes>) -> Option<String> {
    // An access point SSID could be random bytes and not a UTF-8 encoded string
    std::str::from_utf8(&ssid?).ok().map(str::to_owned)
}

fn find_any_wifi_device(client: &Client) -> Result<DeviceWifi> {
    for device in client.devices() {
        if device.device_type() == DeviceType::Wifi && device.state() != DeviceState::Unmanaged {
            return Ok(device.downcast().unwrap());
        }
    }

    bail!("Failed to find a managed WiFi device")
}

async fn create_client() -> Result<Client> {
    let client = Client::new_async_future()
        .await
        .context("Failed to create NetworkManager client")?;

    if !client.is_nm_running() {
        return Err(anyhow!("NetworkManager daemon is not running"));
    }

    Ok(client)
}

async fn delete_exising_wifi_connect_ap_profile(ssid: &str) -> Result<()> {
    let client = create_client().await?;

    let connections = client.connections();

    for connection in connections {
        let c = connection.clone().upcast::<nm::Connection>();
        if is_access_point_connection(&c) && is_same_ssid(&c, ssid) {
            println!(
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
