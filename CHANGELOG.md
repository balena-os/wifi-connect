* Removed bind and udhcpd and used dnsmasq instead
* Removed all iptable logic, dnsmasq replies to DNS requests properly
* Removed wireless-tools npm module
* Use DBus to talk to connman and systemd on the host
* Switched to ip instead of ifconfig
* Made everything Promise based
* Correctly handle `rfkill`ed interfaces
* Allow scanning for networks even while in AP mode
* Maked configuration purely environment variable based
* Persist configuration files atomically and durably
* Only start node process if setup hasn't completed
* Use connman API to find out if connection succeeded
* Make wifi interface configurable
