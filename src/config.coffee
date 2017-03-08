module.exports =
	ssid: process.env.PORTAL_SSID or 'ResinAP'
	passphrase: process.env.PORTAL_PASSPHRASE
	iface: process.env.PORTAL_INTERFACE or 'wlan0'
	gateway: process.env.PORTAL_GATEWAY or '192.168.42.1'
	dhcpRange: process.env.PORTAL_DHCP_RANGE or '192.168.42.2,192.168.42.254'
	connmanConfig: process.env.PORTAL_CONNMAN_CONFIG or '/host/var/lib/connman/network.config'
	persistentConfig: process.env.PORTAL_PERSISTENT_CONFIG or '/data/network.config'
	connectTimeout: process.env.CONNECT_TIMEOUT or 15000
