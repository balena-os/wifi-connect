#!/usr/bin/env bash

export DBUS_SYSTEM_BUS_ADDRESS=unix:path=/host/run/dbus/system_bus_socket

# ------------------------------------------------------------------------------
# Helper: check if we have internet connectivity
# ------------------------------------------------------------------------------
has_internet() {
    nmcli -t g 2>/dev/null | grep -q full || \
        ping -c 1 -W 3 8.8.8.8 >/dev/null 2>&1
}

# ------------------------------------------------------------------------------
# Get connection names of saved WiFi networks that are currently visible (in range)
# ------------------------------------------------------------------------------
get_saved_wifi_visible() {
    local visible_ssids
    visible_ssids=$(nmcli -t -f SSID dev wifi list 2>/dev/null | awk -F: '{print $NF}' | sort -u)
    [ -z "$visible_ssids" ] && return

    local line conn_name conn_ssid conn_type
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        conn_type="${line##*:}"           # last colon-separated field
        [[ "$conn_type" = "802-11-wireless" || "$conn_type" = "wifi" ]] || continue
        conn_name="${line%:*}"             # everything before last colon
        conn_ssid=$(nmcli -g 802-11-wireless.ssid connection show "$conn_name" 2>/dev/null)
        [ -n "$conn_ssid" ] || continue
        if echo "$visible_ssids" | grep -qFx "$conn_ssid"; then
            echo "$conn_name"
        fi
    done <<< "$(nmcli -t -f NAME,TYPE connection show 2>/dev/null)"
}

# ------------------------------------------------------------------------------
# Try to connect to saved WiFi networks that are currently visible (no internet)
# Returns 0 if we got internet, 1 otherwise
# ------------------------------------------------------------------------------
try_saved_networks() {
    local attempts=${RECONNECT_ATTEMPTS:-2}
    local delay=${RECONNECT_DELAY:-5}
    local conn

    while [ "$attempts" -gt 0 ]; do
        for conn in $(get_saved_wifi_visible); do
            printf 'Trying saved network: %s\n' "$conn"
            if nmcli connection up "$conn" 2>/dev/null; then
                sleep "$delay"
                if has_internet; then
                    printf 'Connected to %s with internet\n' "$conn"
                    return 0
                fi
            fi
        done
        attempts=$((attempts - 1))
        [ "$attempts" -gt 0 ] && sleep "$delay"
    done
    return 1
}

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

# If no internet, try connecting to saved networks that are currently visible
if ! has_internet; then
    printf 'No internet. Trying saved networks that are in range...\n'
    try_saved_networks || true
fi

# Check if we have internet (after trying saved networks)
if has_internet; then
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
    if [ ${LAUNCH_APP:-1} -eq 1 ]; then
        printf 'Starting WiFi Connect\n' # AP mode by default
        # Build portal SSID: if BOT_ID exists, prepend to PORTAL_SSID; else use PORTAL_SSID if set
        portal_ssid=""
        if [ -n "$PORTAL_SSID" ]; then
            portal_ssid="${BOT_ID:+"${BOT_ID}-"}${PORTAL_SSID}"
        fi
        ./wifi-connect ${portal_ssid:+--portal-ssid "$portal_ssid"}
    else
        printf 'Skipping WiFi Connect (LAUNCH_APP=0)\n'
    fi
fi

# Start your application here.
sleep infinity
