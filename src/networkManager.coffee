Promise = require 'bluebird'
DBus = require './dbus-promise'
_ = require 'lodash'

dbus = new DBus()

bus = dbus.getBus('system')

systemd = require './systemd'

SERVICE = 'org.freedesktop.NetworkManager'

# This allows us to say, if there IS a connection TYPE other than '802-3-ethernet' then network manager has been set up previously.
WHITE_LIST = ['802-3-ethernet']

NM_STATE_CONNECTED_GLOBAL = 70
NM_DEVICE_TYPE_WIFI = 2
NM_CONNECTIVITY_LIMITED = 4
NM_CONNECTIVITY_FULL = 5

exports.start = ->
	systemd.start('NetworkManager.service')

exports.stop = ->
	systemd.stop('NetworkManager.service')

exports.isSetup = ->
	getConnections()
	.map(validateConnection)
	.then (results) ->
		return true in results

exports.setCredentials = (ssid, passphrase) ->
	connection = {
		'802-11-wireless': {
			ssid: _.invokeMap(ssid, 'charCodeAt')
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

	console.log('Saving connection')
	console.log(connection)

	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager/Settings', 'org.freedesktop.NetworkManager.Settings')
	.then (settings) ->
		settings.AddConnectionAsync(connection)

exports.clearCredentials = ->
	getConnections()
	.map(deleteConnection)

exports.connect  = (timeout) ->
	getDevices()
	.filter(validateDevice)
	.then (validDevices) ->
		if validDevices.length is 0
			throw ('No valid devices found.')
		bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager', 'org.freedesktop.NetworkManager')
		.delay(1000) # Delay needed to avoid "Error: org.freedesktop.NetworkManager.UnknownConnection at Error (native)"
		.then (manager) ->
			manager.ActivateConnectionAsync('/', validDevices[0], '/')
			.then ->
				new Promise (resolve, reject) ->
					handler = (value) ->
						if value == NM_STATE_CONNECTED_GLOBAL
							manager.removeListener('StateChanged', handler)
							resolve()

					# Listen for 'Connected' signals
					manager.on('StateChanged', handler)

					# But try to read in case we registered the event handler
					# after is was already connected
					manager.CheckConnectivityAsync()
					.then (state) ->
						if state == NM_CONNECTIVITY_FULL or state == NM_CONNECTIVITY_LIMITED
							manager.removeListener('StateChanged', handler)
							resolve()

					setTimeout ->
						manager.removeListener('StateChanged', handler)
						reject('Timed out')
					, timeout

getConnections = ->
	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager/Settings', 'org.freedesktop.NetworkManager.Settings')
	.call('ListConnectionsAsync')

getConnection = (connection) ->
	bus.getInterfaceAsync(SERVICE, connection, 'org.freedesktop.NetworkManager.Settings.Connection')

validateConnection = (connection) ->
	getConnection(connection)
	.call('GetSettingsAsync')
	.then (settings) ->
		return settings.connection.type not in WHITE_LIST

deleteConnection = (connection) ->
	getConnection(connection)
	.then (connection) ->
		connection.GetSettingsAsync()
		.then (settings) ->
			if settings.connection.type not in WHITE_LIST
				connection.DeleteAsync()

getDevices = ->
	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager', 'org.freedesktop.NetworkManager')
	.call('GetDevicesAsync')

getDevice = (device) ->
	bus.getInterfaceAsync(SERVICE, device, 'org.freedesktop.NetworkManager.Device')

validateDevice = (device) ->
	getDevice(device)
	.call('getPropertyAsync', 'DeviceType')
	.then (property) ->
		return property == NM_DEVICE_TYPE_WIFI
