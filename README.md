# resin-wifi-connect

A tool to allow wifi configuration to be set via a captive portal.

## How to use this
This is a [resin.io](http://resin.io) application. Check out our [Getting Started](http://docs.resin.io/#/pages/installing/gettingStarted.md) guide if it's your first time using Resin.

This project is meant to be integrated as part of a larger application (that is, _your_ application). An example on how to use this on a Python project can be found [here](https://github.com/resin-io-projects/resin-wifi-connect-python-example).

If you need to add dependencies, add the corresponding statements in the [Dockerfile](./Dockerfile.template) template. You can add the commands that run your app in the [start](./start) script. This app only exits after a successful connection, so if you add your app after [line 3](./start#L3) you ensure that everything happens after wifi is correctly configured.

This is a node.js application, but your app can be any language/framework you want as long as you install it properly - if you need help, check out our [Dockerfile guide](http://docs.resin.io/#/pages/using/dockerfile.md). This project uses a Resin feature called "Dockerfile template": the base image is chosen depending on the architecture, specified by the `%%RESIN_ARCH%%` variable (see [line 1](./Dockerfile.template#L1) in the template).

## Supported boards / dongles
**For the Intel Edison version of this software, check the [edison branch](https://github.com/resin-io/resin-wifi-connect/tree/edison) in this repository.**

This software has been successfully tested on Raspberry Pi's A+ and 2B using the following WiFi dongles:

Dongle                                     | Chip
-------------------------------------------|-------------------
[TP-LINK TL-WN722N](http://bit.ly/1P1MdAG) | Atheros AR9271
[ModMyPi](http://bit.ly/1gY3IHF)           | Ralink RT3070
[ThePiHut](http://bit.ly/1LfkCgZ)          | Ralink RT5370

Given these results, it is probable that most dongles with *Atheros* or *Ralink* chipsets will work.

The following dongles are known **not** to work (as the driver is not friendly with AP mode and Connman):
* Official Raspberry Pi dongle (BCM43143 chip)
* Addon NWU276 (Mediatek MT7601 chip)
* Edimax (Realtek RTL8188CUS chip)
Dongles with similar chipsets will probably not work.

The software is expected to work with other Resin supported boards as long as you use the correct dongles.
Please [contact us](https://resin.io/contact/) or raise [an issue](https://github.com/resin-io/resin-wifi-connect/issues) if you hit any trouble.

## How it works
This app interacts with the Connman connection manager in Resin's base OS. It checks whether WiFi is connected, tries to join the favorite network, and if this fails, it opens an Access Point to which you can connect using a laptop or mobile phone.

The access point's name (SSID) is, by default, "ResinAP". You can change this by changing the "ssid" field in [wifi.json](./src/wifi.json). By default, the network is unprotected, but you can add a WPA2 passphrase by setting the "passphrase" field in the same file. Keep in mind that, once you set a passphrase, you can't go back to an unprotected network on an already provisioned device.
The server for wifi configuration uses port 8080 by default. This can also be configured in wifi.json, but it will be transparent to the user as all web traffic is redirected when in Access Point mode.

These three configurations can also be set with the environment variables `PORTAL_SSID`, `PORTAL_PASSPHRASE` and `PORTAL_PORT`, but keep in mind that the device has to be online to be able to download the new settings (you can use an ethernet cable or a WiFi network to which you've already connected).

When you connect to the access point, any web page you open will be redirected to our captive portal page, where you can select the SSID and passphrase of the WiFi network to connect to. After this, the app will disable the AP and try to connect. If it fails, it will enable the AP for you to try again. If it succeeds, the network will be remembered by Connman as a favorite.
