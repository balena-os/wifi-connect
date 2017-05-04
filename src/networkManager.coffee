Promise = require 'bluebird'
{ spawn, exec } = require 'child_process'
execAsync = Promise.promisify(exec)
DBus = require './dbus-promise'
_ = require 'lodash'

dbus = new DBus()
bus = dbus.getBus('system')

systemd = require './systemd'

SERVICE = 'org.freedesktop.NetworkManager'

# This allows us to say, if there IS an existing connection id that is not in the whitelist then network manager has been set up previously.
WHITE_LIST = ['resin-sample', 'Wired connection 1']

NM_STATE_CONNECTED_GLOBAL = 70
NM_DEVICE_TYPE_WIFI = 2
NM_CONNECTIVITY_LIMITED = 3
NM_CONNECTIVITY_FULL = 4

exports.start = ->
	systemd.start('NetworkManager.service')

exports.stop = ->
	systemd.stop('NetworkManager.service')

exports.ready = ->
	systemd.waitUntilState('NetworkManager.service', 'active')

exports.isSetup = ->
	getConnections()
	.map(isConnectionValid)
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
		}
	}

	if passphrase != ""
		connection = _.merge connection,
			'802-11-wireless-security': {
				'auth-alg': 'open',
				'key-mgmt': 'wpa-psk',
				'psk': passphrase,
			}

	console.log('Saving connection')
	console.log(connection)

	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager/Settings', 'org.freedesktop.NetworkManager.Settings')
	.then (settings) ->
		settings.AddConnectionAsync(connection)
	.then ->
		execAsync('sync')

exports.clearCredentials = ->
	getConnections()
	.map(deleteConnection)

exports.connect  = (timeout) ->
	getDevices()
	.filter(isDeviceValid)
	.then (validDevices) ->
		if validDevices.length is 0
			throw new Error('No valid devices found.')
		getConnections()
		.filter(isConnectionValid)
		.then (validConnections) ->
			if validConnections.length is 0
				throw new Error('No valid connections found.')
			bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager', 'org.freedesktop.NetworkManager')
			.delay(1000) # Delay needed to avoid "Error: org.freedesktop.NetworkManager.UnknownConnection at Error (native)" when activating the connection
			.then (manager) ->
				manager.ActivateConnectionAsync(validConnections[0], validDevices[0], '/')
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
							reject(new Error('Timed out'))
						, timeout

getConnections = ->
	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager/Settings', 'org.freedesktop.NetworkManager.Settings')
	.call('ListConnectionsAsync')

getConnection = (connection) ->
	bus.getInterfaceAsync(SERVICE, connection, 'org.freedesktop.NetworkManager.Settings.Connection')

deleteConnection = (connection) ->
	getConnection(connection)
	.then (connection) ->
		connection.GetSettingsAsync()
		.then (settings) ->
			if settings.connection.id not in WHITE_LIST
				connection.DeleteAsync()

isConnectionValid = (connection) ->
	getConnection(connection)
	.call('GetSettingsAsync')
	.then (settings) ->
		return settings.connection.id not in WHITE_LIST

getDevices = ->
	bus.getInterfaceAsync(SERVICE, '/org/freedesktop/NetworkManager', 'org.freedesktop.NetworkManager')
	.call('GetDevicesAsync')

getDevice = (device) ->
	bus.getInterfaceAsync(SERVICE, device, 'org.freedesktop.NetworkManager.Device')

isDeviceValid = (device) ->
	getDevice(device)
	.call('getPropertyAsync', 'DeviceType')
	.then (property) ->
		return property == NM_DEVICE_TYPE_WIFI
