```
Wifi Connect v3.0.0 now available!
```

# Intro
Resin Wifi Connect is a utility for dynamically setting the Wifi configuration on a device via a captive portal. If the device's WiFi credentials have not been previously configured, or if the device cannot connect using the given credentials, Wifi Connect will create a wireless access point. New Wifi credentials can be specified by connecting to the access point with a laptop or mobile phone.


## How to use this
Wifi Connect is designed to be integrated with a [resin.io](http://resin.io) application. (New to resin.io? Check out the [Getting Started Guide](http://docs.resin.io/#/pages/installing/gettingStarted.md).) This integration is accomplished through the use of two shared files:
- The [Dockerfile template](./Dockerfile.template) manages dependencies. The example included here has everything necessary for Wifi Connect. Application dependencies need to be added. For help with Dockerfiles, take a look at this [guide](https://docs.resin.io/deployment/dockerfile/).
- The [start script](./start) should contain the commands that run the application. Adding these commands after [line 5](./start#L5) will ensure that everything kicks off after Wifi is correctly configured. 
An example of using Wifi Connect in a Python project can be found [here](https://github.com/resin-io-projects/resin-wifi-connect-example).

## How it works
Wifi Connect interacts with Network Manager, which runs in the device's host OS. It opens an access point if a Wifi connection cannot be made, and connecting to this access point with a laptop or mobile phone allows new Wifi credentials to be configured.

The access point SSID is, by default, `ResinAP`. It can be changed by setting the `PORTAL_SSID` environment variable (see [this guide](https://docs.resin.io/management/env-vars/) for how to manage environment variables). By default, the network is unprotected, but a WPA2 passphrase can be added by setting the `PORTAL_PASSPHRASE` environment variable.

After connecting to the access point, any web page will redirect to the captive portal, which provides the option to select a Wifi SSID and passphrase. When these have been entered, Wifi Connect will disable the access point and try to connect to the network. If the connection fails, it will enable the access point for another attempt. If it succeeds, the configuration will be saved by Network Manager.

By default, Wifi Connect will not attempt to enter access point mode if a successful network connection has been made before. If the device is moved to a new network, it will continue trying to connect to the previous network. The user application is responsible for specifying an appropriate condition for returning to access point mode. This could be "offline for more than 1 day", "user pushed the reset button", or any other actionable state. To re-enter access point mode, the application should run the command `resin-wifi-connect --clear=true`.

For a complete list of command line arguments and environment variables check out our [command line arguments](https://github.com/resin-io/resin-wifi-connect/wiki/Command-Line-Arguments) guide.

## State flow diagram
![State flow diagram](./images/flow.png?raw=true)


## Supported boards / dongles
Wifi Connect has been successfully tested using the following Wifi dongles:

Dongle                                     | Chip
-------------------------------------------|-------------------
[TP-LINK TL-WN722N](http://bit.ly/1P1MdAG) | Atheros AR9271
[ModMyPi](http://bit.ly/1gY3IHF)           | Ralink RT3070
[ThePiHut](http://bit.ly/1LfkCgZ)          | Ralink RT5370

It has also been successfully tested with the onboard Wifi on a Raspberry Pi 3.

Given these results, it is probable that most dongles with *Atheros* or *Ralink* chipsets will work.

The following dongles are known **not** to work (as the driver is not friendly with access point mode or Network Manager):

* Official Raspberry Pi dongle (BCM43143 chip)
* Addon NWU276 (Mediatek MT7601 chip)
* Edimax (Realtek RTL8188CUS chip)

Dongles with similar chipsets will probably not work.

Wifi Connect is expected to work with all resin.io supported boards as long as they have the compatible dongles.

Please [contact us](https://resin.io/community/) or raise [an issue](https://github.com/resin-io/resin-wifi-connect/issues) if you hit any trouble.

## FAQ
* *What is the state of Linux networking before the start script is executed?*
If the device is plugged in over Ethernet it will have an internet connection. If the device is using Wifi only, it will not have a connection until the start script has completed.

* *How long will Wifi Connect attempt to connect with a given configuration?*
The connection timeout is set to 15 seconds.
