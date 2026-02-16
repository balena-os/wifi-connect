# WiFi Connect Command Line Arguments

## Flags

*   **-h, --help**

    Prints help information

*   **-V, --version**

    Prints version information

## Options

Command line options have environment variable counterpart. If both a command line option and its environment variable counterpart are defined, the command line option will take higher precedence.

*   **-d, --portal-dhcp-range** dhcp_range, **$PORTAL_DHCP_RANGE**

    DHCP range of the captive portal WiFi network

    Default: _192.168.42.2,192.168.42.254_

*   **-g, --portal-gateway** gateway, **$PORTAL_GATEWAY**

    Gateway of the captive portal WiFi network

    Default: _192.168.42.1_

*   **-o, --portal-listening-port** listening_port, **$PORTAL_LISTENING_PORT**

    Listening port of the captive portal web server

    Default: _80_

*   **-i, --portal-interface** interface, **$PORTAL_INTERFACE**

    Wireless network interface to be used by WiFi Connect

*   **-p, --portal-passphrase** passphrase, **$PORTAL_PASSPHRASE**

    WPA2 Passphrase of the captive portal WiFi network

    Default: _no passphrase_

*   **-s, --portal-ssid** ssid, **$PORTAL_SSID**

    SSID of the captive portal WiFi network

    Default: _WiFi Connect_

*   **-a, --activity-timeout** timeout, **$ACTIVITY_TIMEOUT**

    Exit if no activity for the specified timeout (seconds)

    Default: _0 - no timeout_

*   **-u, --ui-directory** ui_directory, **$UI_DIRECTORY**

    Web UI directory location

    Default: _ui_

*   **--no-ap**

    Serve the UI without creating an access point (station mode). The web interface listens on all interfaces (0.0.0.0) instead of only the captive portal gateway. Useful when the device is already connected to a network and you want to configure WiFi from another device on the same network.

    Environment variable: **$NO_AP** (set to `1` or `true` to enable)

*   **--auth-user** user, **$AUTH_USER**

    HTTP Basic Auth username for protecting the web UI. Only used when `--no-ap` is enabled. Requires `--auth-password` to be set as well. When both are configured, all requests to the UI require authentication.

*   **--auth-password** password, **$AUTH_PASSWORD**

    HTTP Basic Auth password for protecting the web UI. Only used when `--no-ap` is enabled. Requires `--auth-user` to be set as well.
