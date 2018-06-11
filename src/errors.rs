use network_manager;

use network;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Recv(::std::sync::mpsc::RecvError);
        SendNetworkCommand(::std::sync::mpsc::SendError<network::NetworkCommand>);
        Nix(::nix::Error);
    }

    links {
        NetworkManager(network_manager::errors::Error, network_manager::errors::ErrorKind);
    }

    errors {
        RecvAccessPointSSIDs {
            description("Receiving access point SSIDs failed")
        }

        SendAccessPointSSIDs {
            description("Sending access point SSIDs failed")
        }

        SerializeAccessPointSSIDs {
            description("Serializing access point SSIDs failed")
        }

        RecvNetworkCommand {
            description("Receiving network command failed")
        }

        SendNetworkCommandActivate {
            description("Sending NetworkCommand::Activate failed")
        }

        SendNetworkCommandConnect {
            description("Sending NetworkCommand::Connect failed")
        }

        DeviceByInterface(interface: String) {
            description("Cannot find network device with interface name")
            display("Cannot find network device with interface name '{}'", interface)
        }

        NotAWiFiDevice(interface: String) {
            description("Not a WiFi device")
            display("Not a WiFi device: {}", interface)
        }

        NoWiFiDevice {
            description("Cannot find a WiFi device")
        }

        NoAccessPoints {
            description("Getting access points failed")
        }

        CreateCaptivePortal {
            description("Creating the captive portal failed")
        }

        StopAccessPoint {
            description("Stopping the access point failed")
        }

        DeleteAccessPoint {
            description("Deleting access point connection profile failed")
        }

        StartHTTPServer(address: String, reason: String) {
            description("Cannot start HTTP server")
            display("Cannot start HTTP server on '{}': {}", address, reason)
        }

        StartActiveNetworkManager {
            description("Starting the NetworkManager service with active state failed")
        }

        StartNetworkManager {
            description("Starting the NetworkManager service failed")
        }

        Dnsmasq {
            description("Spawning dnsmasq failed")
        }

        BlockExitSignals {
            description("Blocking exit signals failed")
        }

        TrapExitSignals {
            description("Trapping exit signals failed")
        }

        RootPrivilegesRequired(app: String) {
            description("Root privileges required")
            display("You need root privileges to run {}", app)
        }
    }
}

pub fn exit_code(e: &Error) -> i32 {
    match *e.kind() {
        ErrorKind::Dnsmasq => 3,
        ErrorKind::RecvAccessPointSSIDs => 4,
        ErrorKind::SendAccessPointSSIDs => 5,
        ErrorKind::SerializeAccessPointSSIDs => 6,
        ErrorKind::RecvNetworkCommand => 7,
        ErrorKind::SendNetworkCommandActivate => 8,
        ErrorKind::SendNetworkCommandConnect => 9,
        ErrorKind::DeviceByInterface(_) => 10,
        ErrorKind::NotAWiFiDevice(_) => 11,
        ErrorKind::NoWiFiDevice => 12,
        ErrorKind::NoAccessPoints => 13,
        ErrorKind::CreateCaptivePortal => 14,
        ErrorKind::StopAccessPoint => 15,
        ErrorKind::DeleteAccessPoint => 16,
        ErrorKind::StartHTTPServer(_, _) => 17,
        ErrorKind::StartActiveNetworkManager => 18,
        ErrorKind::StartNetworkManager => 19,
        ErrorKind::BlockExitSignals => 21,
        ErrorKind::TrapExitSignals => 22,
        ErrorKind::RootPrivilegesRequired(_) => 23,
        _ => 1,
    }
}
