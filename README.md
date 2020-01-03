<h1 align="center"><img width="460" src="https://github.com/balena-io/wifi-connect/raw/master/docs/images/wifi-connect.png" /></h1>

> Easy WiFi setup for Linux devices from your mobile phone or laptop

WiFi Connect is a utility for dynamically setting the WiFi configuration on a Linux device via a captive portal. WiFi credentials are specified by connecting with a mobile phone or laptop to the access point that WiFi Connect creates.

[![Current Release](https://img.shields.io/github/release/balena-io/wifi-connect.svg?style=flat-square)](https://github.com/balena-io/wifi-connect/releases/latest)
[![CircleCI status](https://img.shields.io/circleci/project/github/balena-io/wifi-connect.svg?style=flat-square)](https://circleci.com/gh/balena-io/wifi-connect)
[![License](https://img.shields.io/github/license/balena-io/wifi-connect.svg?style=flat-square)](https://github.com/balena-io/wifi-connect/blob/master/LICENSE)
[![Issues](https://img.shields.io/github/issues/balena-io/wifi-connect.svg?style=flat-square)](https://github.com/balena-io/wifi-connect/issues)

<div align="center">
  <sub>an open source :satellite: project by <a href="https://balena.io">balena.io</a></sub>
</div>

***

[**Download**][DOWNLOAD] | [**How it works**](#how-it-works) | [**Installation**](#installation) | [**Support**](#support) | [**Roadmap**][MILESTONES]

[DOWNLOAD]: https://github.com/balena-io/wifi-connect/releases/latest
[MILESTONES]: https://github.com/balena-io/wifi-connect/milestones

![How it works](./docs/images/how-it-works.png?raw=true)

How it works
------------

WiFi Connect interacts with NetworkManager, which should be the active network manager on the device's host OS.

### 1. Advertise: Device Creates Access Point

WiFi Connect detects available WiFi networks and opens an access point with a captive portal. Connecting to this access point with a mobile phone or laptop allows new WiFi credentials to be configured.

### 2. Connect: User Connects Phone to Device Access Point

Connect to the opened access point on the device from your mobile phone or laptop. The access point SSID is, by default, `WiFi Connect`. It can be changed by setting the `--portal-ssid` command line argument or the `PORTAL_SSID` environment variable (see [this guide](https://balena.io/docs/management/env-vars/) for how to manage environment variables when running on top of balenaOS). By default, the network is unprotected, but a WPA2 passphrase can be added by setting the `--portal-passphrase` command line argument or the `PORTAL_PASSPHRASE` environment variable.

### 3. Portal: Phone Shows Captive Portal to User

After connecting to the access point from a mobile phone, it will detect the captive portal and open its web page. Opening any web page will redirect to the captive portal as well.

### 4. Credentials: User Enters Local WiFi Network Credentials on Phone

The captive portal provides the option to select a WiFi SSID from a list with detected WiFi networks and enter a passphrase for the desired network.

### 5. Connected!: Device Connects to Local WiFi Network

When the network credentials have been entered, WiFi Connect will disable the access point and try to connect to the network. If the connection fails, it will enable the access point for another attempt. If it succeeds, the configuration will be saved by NetworkManager.

---

For a complete list of command line arguments and environment variables check out our [command line arguments](./docs/command-line-arguments.md) guide.

The full application flow is illustrated in the [state flow diagram](./docs/state-flow-diagram.md).

***

Installation
------------

WiFi Connect is designed to work on systems like Raspbian or Debian, or run in a docker container on top of balenaOS.

### Raspbian/Debian Stretch

WiFi Connect depends on NetworkManager, but by default Raspbian Stretch uses dhcpcd as a network manager. The provided installation shell script disables dhcpcd, installs NetworkManager as the active network manager and downloads and installs WiFi Connect.

Run the following in your terminal, then follow the onscreen instructions:

`bash <(curl -L https://github.com/balena-io/wifi-connect/raw/master/scripts/raspbian-install.sh)`

### balenaOS

WiFi Connect can be integrated with a [balena.io](http://balena.io) application. (New to balena.io? Check out the [Getting Started Guide](https://balena.io/docs/#/pages/installing/gettingStarted.md).) This integration is accomplished through the use of two shared files:
- The [Dockerfile template](./Dockerfile.template) manages dependencies. The example included here has everything necessary for WiFi Connect. Application dependencies need to be added. For help with Dockerfiles, take a look at this [guide](https://balena.io/docs/deployment/dockerfile/).
- The [start script](./scripts/start.sh) should contain the commands that run the application. Adding these commands at the end of the script will ensure that everything kicks off after WiFi is correctly configured. 
An example of using WiFi Connect in a Python project can be found [here](https://github.com/balena-io-projects/balena-wifi-connect-example).

### balenaOS: multicontainer app

To use WiFi Connect on a multicontainer app you need to:
- Set container network mode to host
- Enable DBUS by adding the required label and environment variable (more on [balenaOS dbus](https://www.balena.io/docs/learn/develop/runtime/#dbus-communication-with-host-os))
- Grant the container Network Admin capabilities

Your `docker-compose.yml` file should look like this:
```yaml
version: "2.1"

services:
    wifi-connect:
        build: ./wifi-connect
        network_mode: "host"
        labels:
            io.balena.features.dbus: '1'
        cap_add:
            - NET_ADMIN
        environment:
            DBUS_SYSTEM_BUS_ADDRESS: "unix:path=/host/run/dbus/system_bus_socket"
    ...
```

***

Supported boards / dongles
--------------------------

WiFi Connect has been successfully tested using the following WiFi dongles:

Dongle                                     | Chip
-------------------------------------------|-------------------
[TP-LINK TL-WN722N](http://bit.ly/1P1MdAG) | Atheros AR9271
[ModMyPi](http://bit.ly/1gY3IHF)           | Ralink RT3070
[ThePiHut](http://bit.ly/1LfkCgZ)          | Ralink RT5370

It has also been successfully tested with the onboard WiFi on a Raspberry Pi 3.

Given these results, it is probable that most dongles with *Atheros* or *Ralink* chipsets will work.

The following dongles are known **not** to work (as the driver is not friendly with access point mode or NetworkManager):

* Official Raspberry Pi dongle (BCM43143 chip)
* Addon NWU276 (Mediatek MT7601 chip)
* Edimax (Realtek RTL8188CUS chip)

Dongles with similar chipsets will probably not work.

WiFi Connect is expected to work with all balena.io [supported boards](https://www.balena.io/docs/reference/hardware/devices/) as long as they have the [compatible dongles](https://www.balena.io/docs/reference/hardware/wifi-dongles/).

***

Support
-------

If you're having any problem, please [raise an issue](https://github.com/balena-io/wifi-connect/issues/new) on GitHub or [contact us](https://balena.io/community/), and the balena.io team will be happy to help.

***

License
-------

WiFi Connect is free software, and may be redistributed under the terms specified in
the [license](https://github.com/balena-io/wifi-connect/blob/master/LICENSE).
