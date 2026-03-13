#!/usr/bin/env bash

export DBUS_SYSTEM_BUS_ADDRESS=unix:path=/host/run/dbus/system_bus_socket

# Optional step - it takes couple of seconds (or longer) to establish a WiFi connection
# sometimes. In this case, following checks will fail and wifi-connect
# will be launched even if the device will be able to connect to a WiFi network.
# If this is your case, you can wait for a while and then check for the connection.
# Configurable via START_SLEEP env var (default: 25 seconds).
sleep ${START_SLEEP:-25}

# Choose a condition for running WiFi Connect according to your use case:

# 1. Is there a default gateway?
# ip route | grep default

# 2. Is there Internet connectivity?
# nmcli -t g | grep full

# 3. Is there Internet connectivity via a google ping?
# wget --spider http://google.com 2>&1

# Query the network manager list of networks to cache the available access points
nmcli -t -f SSID dev wifi list > /usr/src/app/access-points.txt

# 4. Is there an active WiFi connection?
echo "Active WiFi connection:"
iwgetid -r

# Check if there is an active WiFi connection
if [ $? -eq 0 ]; then
    if [ ${LAUNCH_IF_CONNECTED:-1} -eq 1 ]; then
        # check if NO_AP_MODE_PASSWORD is set
        if [ -z "$NO_AP_MODE_PASSWORD" ]; then
            echo "NO_AP_MODE_PASSWORD is not set, not running wifi-connect in no-ap mode"
        else
            ./wifi-connect --no-ap --auth-user robot --auth-password $NO_AP_MODE_PASSWORD
        fi
    else
        printf 'Skipping WiFi Connect\n'
    fi
else
    printf 'Starting WiFi Connect\n' # AP mode by default
    # Build portal SSID: if BOT_ID exists, prepend to PORTAL_SSID; else use PORTAL_SSID if set
    portal_ssid=""
    if [ -n "$PORTAL_SSID" ]; then
        portal_ssid="${BOT_ID:+"${BOT_ID}-"}${PORTAL_SSID}"
    fi
    ./wifi-connect ${portal_ssid:+--portal-ssid "$portal_ssid"}
fi

# Start your application here.
sleep infinity
