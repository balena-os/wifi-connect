Promise = require 'bluebird'
DBus = require './dbus-promise'

dbus = new DBus()

bus = dbus.getBus('system')

systemd = require './systemd'

SERVICE = 'org.freedesktop.NetworkManager'
WHITE_LIST = ['resin-vpn', 'eth0']

exports.start = ->
	systemd.start('NetworkManager.service')

exports.stop = ->
	systemd.stop('NetworkManager.service')

exports.isSetup = ->
	getConnections()
	.then (connections) ->
		buffer = []
		for connection in connections
			buffer.push(validateConnection(connection))

		Promise.all(buffer)
		.then (results) ->
			return true in results

exports.setCredentials = (ssid, passphrase) ->
	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager/Settings', 'org.freedesktop.NetworkManager.Settings')
	.then (manager) ->
		connection = {
			'802-11-wireless': {
				ssid: unpack(ssid)
			},
			connection: {
				id: ssid,
				type: '802-11-wireless',
			},
			'802-11-wireless-security': {
				'auth-alg': 'open',
				'key-mgmt': 'wpa-psk',
				'psk': passphrase,
			}
		}

		manager.AddConnectionAsync(connection)

exports.clearCredentials = ->
	getConnections()
	.then (connections) ->
		buffer = []
		for connection in connections
			buffer.push(deleteConnection(connection))

		Promise.all(buffer)

exports.connect  = (timeout) ->
	getDevices()
	.then (devices) ->
		buffer = []
		for device in devices
			buffer.push(validateDevice(device, 2))

		_manager = null
		Promise.all(buffer)
		.then (result) ->
			bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager', 'org.freedesktop.NetworkManager')
			.then (manager) ->
				_manager = manager
				_manager.ActivateConnectionAsync('/', devices[result.indexOf(true)], '/')
			.then ->
				new Promise (resolve, reject, onCancel) ->
					handler = (value) ->
						if value == 70
							_manager.removeListener('StateChanged', handler)
							resolve()

					_manager.on('StateChanged', handler)

					_manager.CheckConnectivityAsync()
					.then (state) ->
						if state == 5
							_manager.removeListener('StateChanged', handler)
							resolve()

					setTimeout ->
						_manager.removeListener('StateChanged', handler)
						reject()
					, timeout

getConnections = ->
	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager/Settings', 'org.freedesktop.NetworkManager.Settings')
	.then (settings) ->
		settings.ListConnectionsAsync()

getConnection = (connection) ->
	bus.getInterfaceAsync(SERVICE, connection, 'org.freedesktop.NetworkManager.Settings.Connection')

validateConnection = (connection) ->
	getConnection(connection)
	.then (connection) ->
		connection.GetSettingsAsync()
		.then (settings) ->
			return settings.connection.id in WHITE_LIST

deleteConnection = (connection) ->
	_connection = null
	getConnection(connection)
	.then (connection) ->
		_connection = connection
		connection.GetSettingsAsync()
		.then (settings) ->
			if settings.connection.id not in WHITE_LIST
				_connection.DeleteAsync()

getDevices = ->
	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager', 'org.freedesktop.NetworkManager')
	.then (manager) ->
		manager.GetDevicesAsync()

getDevice = (device) ->
	bus.getInterfaceAsync(SERVICE, device, 'org.freedesktop.NetworkManager.Device')

validateDevice = (device, type) ->
	getDevice(device)
	.then (device) ->
		device.getPropertyAsync('DeviceType')
		.then (property) ->
			return property == type

unpack = (str) ->
	bytes = []
	i = 0
	n = str.length
	while i < n
		char = str.charCodeAt(i)
		bytes.push(char)
		i++
	return bytes
