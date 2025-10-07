#!/bin/bash

set -u
set -o pipefail  # Make pipelines return failure if any command fails

trap "exit 1" TERM
export TOP_PID=$$

: "${WFC_REPO:=balena-os/wifi-connect}"
: "${WFC_INSTALL_ROOT:=/usr/local}"

SCRIPT='raspbian-install.sh'
NAME='WiFi Connect Raspbian Installer'

INSTALL_BIN_DIR="$WFC_INSTALL_ROOT/sbin"
INSTALL_UI_DIR="$WFC_INSTALL_ROOT/share/wifi-connect/ui"

RELEASE_URL="https://api.github.com/repos/$WFC_REPO/releases/latest"

CONFIRMATION=true

usage() {
    cat 1>&2 <<EOF
$NAME 2.0.0 (2025-10-07)

USAGE:
    $SCRIPT [FLAGS]

FLAGS:
    -y                      Disable confirmation prompt
    -h, --help              Prints help information
EOF
}

main() {
    for arg in "$@"; do
        case "$arg" in
            -h|--help)
                usage
                exit 0
                ;;
            -y)
                CONFIRMATION=false
                ;;
            *)
                ;;
        esac
    done

    need_cmd id
    need_cmd curl
    need_cmd systemctl
    need_cmd apt-get
    need_cmd grep
    need_cmd mktemp
    need_cmd uname

    check_os_version

    install_wfc

    activate_network_manager

    say "Run 'wifi-connect --help' for available options"
}

check_os_version() {
    local _version=""

    if [ -f /etc/os-release ]; then
        _version=$(grep -oP 'VERSION="\K[^"]+' /etc/os-release)
    fi

    if [ "$_version" == "8 (jessie)" ]; then
        err "Distributions based on Debian 8 (jessie) are not supported"
    fi
}

detect_architecture() {
    local _arch
    _arch=$(uname -m)

    case "$_arch" in
        aarch64|arm64)
            echo "aarch64-unknown-linux-gnu"
            ;;
        armv7l|armv6l)
            echo "armv7-unknown-linux-gnueabihf"
            ;;
        x86_64|amd64)
            echo "x86_64-unknown-linux-gnu"
            ;;
        i686|i386)
            echo "i686-unknown-linux-gnu"
            ;;
        *)
            err "Unsupported architecture: $_arch"
            ;;
    esac
}

activate_network_manager() {
    if [ "$(service_load_state NetworkManager)" = "not-found" ]; then
        say 'NetworkManager is not installed'

        confirm_installation

        # Do not install NetworkManager over running dhcpcd to avoid clashes

        say 'Downloading NetworkManager...'

        ensure sudo apt-get update

        ensure sudo apt-get install -y -d network-manager

        disable_dhcpcd

        say 'Installing NetworkManager...'

        ensure sudo apt-get install -y network-manager

        ensure sudo apt-get clean
    else
        say 'NetworkManager is already installed'

        if [ "$(service_active_state NetworkManager)" = "active" ]; then
            say 'NetworkManager is already active'
        else
            confirm_installation

            disable_dhcpcd

            say 'Activating NetworkManager...'

            ensure sudo systemctl enable NetworkManager

            ensure sudo systemctl start NetworkManager
        fi
    fi

    if [ ! "$(service_active_state NetworkManager)" = "active" ]; then
        err 'Cannot activate NetworkManager'
    fi
}

disable_dhcpcd() {
    if [ "$(service_active_state dhcpcd)" = "active" ]; then
        say 'Deactivating and disabling dhcpcd...'

        ensure sudo systemctl stop dhcpcd

        ensure sudo systemctl disable dhcpcd

        if [ "$(service_active_state dhcpcd)" = "active" ]; then
            err 'Cannot deactivate dhcpcd'
        else
            say 'dhcpcd successfully deactivated and disabled'
        fi
    else
        say 'dhcpcd is not active'
    fi
}

service_load_state() {
    ensure systemctl -p LoadState --value show "$1"
}

service_active_state() {
    ensure systemctl -p ActiveState --value show "$1"
}

confirm_installation() {
    if [ "$CONFIRMATION" = false ]; then
        return
    fi

    printf '\33[1;36m%s:\33[0m ' "$NAME"

    read -r -p "Continue to install NetworkManager and disable dhcpcd? [y/N] " response
    response=${response,,}  # convert to lowercase
    if [[ ! $response =~ ^(yes|y)$ ]]; then
        exit 0
    fi
}

install_wfc() {
    local _arch
    local _binary_regex
    local _ui_regex='browser_download_url": "\K.*wifi-connect-ui\.tar\.gz'
    local _binary_url
    local _ui_url
    local _wfc_version
    local _download_dir
    local _ui_download_dir

    say "Detecting architecture..."
    
    _arch=$(detect_architecture)
    
    say "Detected architecture: $_arch"
    
    _binary_regex="browser_download_url\": \"\\K.*${_arch}\\.tar\\.gz"

    say "Retrieving latest release from $RELEASE_URL..."

    local _release_data
    _release_data=$(ensure curl -s "$RELEASE_URL")

    _binary_url=$(echo "$_release_data" | grep -oP "$_binary_regex" | head -n 1)
    _ui_url=$(echo "$_release_data" | grep -oP "$_ui_regex" | head -n 1)

    if [ -z "$_binary_url" ]; then
        err "Could not find binary download URL for architecture: $_arch"
    fi

    if [ -z "$_ui_url" ]; then
        err "Could not find UI download URL"
    fi

    say "Downloading and extracting binary from $_binary_url..."

    _download_dir=$(ensure mktemp -d)

    # Download binary and extract
    if ! curl -Ls "$_binary_url" | tar -xz -C "$_download_dir"; then
        err "Failed to download or extract binary"
    fi

    if [ ! -f "$_download_dir/wifi-connect" ]; then
        err "wifi-connect binary not found in downloaded archive"
    fi

    ensure sudo mv "$_download_dir/wifi-connect" "$INSTALL_BIN_DIR"

    say "Downloading and extracting UI from $_ui_url..."

    # Create separate directory for UI download
    _ui_download_dir=$(ensure mktemp -d)

    # Download UI and extract
    if ! curl -Ls "$_ui_url" | tar -xz -C "$_ui_download_dir"; then
        err "Failed to download or extract UI"
    fi

    # Check what was actually extracted
    if [ -d "$_ui_download_dir/ui" ]; then
        # UI is in a subdirectory
        ensure sudo mkdir -p "$(dirname "$INSTALL_UI_DIR")"
        ensure sudo rm -rf "$INSTALL_UI_DIR"
        ensure sudo mv "$_ui_download_dir/ui" "$INSTALL_UI_DIR"
    elif [ -f "$_ui_download_dir/index.html" ]; then
        # UI files are in the root of the archive
        ensure sudo mkdir -p "$INSTALL_UI_DIR"
        ensure sudo rm -rf "$INSTALL_UI_DIR"
        ensure sudo mkdir -p "$INSTALL_UI_DIR"
        ensure sudo cp -r "$_ui_download_dir"/* "$INSTALL_UI_DIR/"
    else
        err "Could not find UI files in downloaded archive. Contents: $(ls -la "$_ui_download_dir")"
    fi

    # Cleanup both temp directories
    ensure rm -rf "$_download_dir"
    ensure rm -rf "$_ui_download_dir"

    _wfc_version=$(ensure wifi-connect --version)

    say "Successfully installed $_wfc_version"
}

say() {
    printf '\33[1m%s:\33[0m %s\n' "$NAME" "$1"
}

err() {
    printf '\33[1;31m%s:\33[0m %s\n' "$NAME" "$1" >&2
    kill -s TERM $TOP_PID
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need '$1' (command not found)"
    fi
}

ensure() {
    "$@"
    if [ $? != 0 ]; then
        err "command failed: $*";
    fi
}

main "$@" || exit 1
