#!/usr/bin/env bash

export DBUS_SYSTEM_BUS_ADDRESS=unix:path=/host/run/dbus/system_bus_socket

# 環境変数DEVICE_NAMEが無ければ環境変数BALENA_DEVICE_NAME_AT_INITを文字列結合する
# 環境変数BALENA_DEVICE_NAME_AT_INITが無ければ文字列"sample"を文字列結合する
wifi_connect_portal_ssid="WiFi-Connect_${DEVICE_NAME:-${BALENA_DEVICE_NAME_AT_INIT:-sample}}"

# Choose a condition for running WiFi Connect according to your use case:

# 1. Is there a default gateway?
# ip route | grep default

# 2. Is there Internet connectivity?
# nmcli -t g | grep full

# 3. Is there Internet connectivity via a google ping?
# wget --spider http://google.com 2>&1

# 4. Is there an active WiFi connection?
iwgetid -r

if [ $? -eq 0 ]; then
    printf 'Skipping WiFi Connect\n'
else
    printf 'Starting WiFi Connect\n'
    ./wifi-connect --portal-ssid "$wifi_connect_portal_ssid"
fi

# Start your application here.
sleep infinity
