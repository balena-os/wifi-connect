# resin-wifi-connect
An app to allow WiFi configuration to be set via a captive portal. First it checks whether WiFi has been previously configured, if it has it attempts to connect to the configured network, if it hasn't it opens an Access Point to which you can connect using a laptop or mobile phone and input new WiFi credentials.


## How to use this
This is a [resin.io](http://resin.io) application. Check out our [Getting Started](http://docs.resin.io/#/pages/installing/gettingStarted.md) guide if it's your first time using resin.io

This project is meant to be integrated as part of a larger application (that is, _your_ application). An example on how to use this on a Python project can be found [here](https://github.com/resin-io-projects/resin-wifi-connect-example).

If you need to add dependencies, add the corresponding statements in the [Dockerfile](./Dockerfile.template) template. You can add the commands that run your app in the [start](./start) script. resin-wifi-connect only exits after a WiFi connection has been correctly configured, so if you add your app after [line 7](./start#L7) you ensure that everything happens after WiFi is correctly configured.

This is a node.js application, but your app can be any language/framework you want as long as you install it properly - if you need help, check out our [Dockerfile guide](https://docs.resin.io/deployment/dockerfile/). This project uses a Resin feature called "Dockerfile template", this means that the base image is chosen depending on the architecture, specified by the `%%RESIN_MACHINE_NAME%%` variable (see [line 1](./Dockerfile.template#L1) in the template).

## How it works
This app interacts with either Connman or Network Manager in Resin's base OS. First it checks whether WiFi has been previously configured, if it has it attempts to connect to the configured network, if it hasn't it opens an Access Point to which you can connect using a laptop or mobile phone and input new WiFi credentials.

The Access Points's name (SSID) is, by default, "ResinAP". You can change this by setting the `PORTAL_SSID` environment variable. By default, the network is unprotected, but you can add a WPA2 passphrase by setting the `PORTAL_PASSPHRASE` environment variable.

When you connect to the Access Point, any web page you open will be redirected to our captive portal page, where you can select the SSID and passphrase of the WiFi network to connect to. After this, the app will disable the Access Point and try to connect. If the connection fails, it will enable the Access Point for you to try again. If it succeeds, the configuration will be saved by Connman or Network Manager.

An important detail is that by default, the project will not attempt to enter Access Point mode if a successful configuration has happened in the past. This means that if you go through the process and then move the device to a different network, it will be trying to connect forever. It is left to the user application to decide which is the appropriate condition to re-enter Access Point mode. This can be "been offline for more than 1 day" or "user pushed the reset button" or something else. To re-enter access point mode, simply run `node src/app.js --clear=true`.

## State flow diagram
![State flow diagram](./images/flow.png?raw=true)


## Supported boards / dongles
This app has been successfully tested using the following WiFi dongles:

Dongle                                     | Chip
-------------------------------------------|-------------------
[TP-LINK TL-WN722N](http://bit.ly/1P1MdAG) | Atheros AR9271
[ModMyPi](http://bit.ly/1gY3IHF)           | Ralink RT3070
[ThePiHut](http://bit.ly/1LfkCgZ)          | Ralink RT5370

The app has also been successfully tested on RaspberryPi 3 with its onboard wifi.

Given these results, it is probable that most dongles with *Atheros* or *Ralink* chipsets will work.

The following dongles are known **not** to work (as the driver is not friendly with Access Point mode, Connman or Network Manager):

* Official Raspberry Pi dongle (BCM43143 chip)
* Addon NWU276 (Mediatek MT7601 chip)
* Edimax (Realtek RTL8188CUS chip)

Dongles with similar chipsets will probably not work.

This app is expected to work with other Resin supported boards as long as you use the correct dongles.

Please [contact us](https://resin.io/community/) or raise [an issue](https://github.com/resin-io/resin-wifi-connect/issues) if you hit any trouble.

## FAQ
* *What is the state of Linux networking before the start script is executed?*
If the device is plugged in over Ethernet it will have an internet connection, if the device is using WiFi only it will not have an internet connection until the start script has completed.

* *How long will this app attempt to connect to a configured connection for?*
The connection timeout is set to 15 seconds.
