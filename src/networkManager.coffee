Promise = require 'bluebird'
DBus = require './dbus-promise'

dbus = new DBus()

bus = dbus.getBus('system')

config = require './config'
fs = Promise.promisifyAll(require('fs'))
systemd = require './systemd'
utils = require './utils'

SERVICE = 'org.freedesktop.NetworkManager'
WIFI_OBJECT = '/org/freedesktop/NetworkManager/Settings'
TECHNOLOGY_INTERFACE = 'org.freedesktop.NetworkManager.Settings'

DEFAULT_CONNECTIONS = 2

exports.start = () ->
	systemd.start('NetworkManager.service')

exports.stop = () -> 
	systemd.stop('NetworkManager.service')

exports.isSetup = () ->
	new Promise (resolve, reject) ->
		getConnections()
		.then (connections) ->
			if connections.length > DEFAULT_CONNECTIONS
				resolve (true)
			else 
				resolve (false)
		.catch (e) ->
			reject (e)

exports.setCredentials = (ssid, passphrase) ->
	data = """
				[service_home_ethernet]
				Type = ethernet
				Nameservers = 8.8.8.8,8.8.4.4

				[service_home_wifi]
				Type = wifi
				Name = #{ssid}
				Passphrase = #{passphrase}
				Nameservers = 8.8.8.8,8.8.4.4

			"""

	console.log('Saving credentials')
	console.log(data)

	return utils.durableWriteFile(config.persistentConfig, data)

exports.clearCredentials = () ->
	new Promise (resolve, reject) ->
		console.log("yoyoyo")
		index = DEFAULT_CONNECTIONS
		getConnections()
		.then (connections) ->
			console.log(index)
			while true
				console.log(index)
				bus.getInterfaceAsync(SERVICE, connections[index], TECHNOLOGY_INTERFACE + ".Connection")
				.then (connection) ->
					console.log("delete")
					connection.DeleteAsync()
				.then ->
					index += 1
					console.log(index)
					if index == connections.length
						console.log("jhjhj")
						resolve
				.catch (e) ->
					console.log(e)
					reject (e)
		.catch (e) ->
			reject (e)

exports.connect  = (timeout) ->
	bus.getInterfaceAsync(SERVICE, WIFI_OBJECT, TECHNOLOGY_INTERFACE)
	.then (wifi) ->
		new Promise (resolve, reject, onCancel) ->
			handler = (name, value) ->
				if name is 'Connected' and value is true
					wifi.removeListener('PropertyChanged', handler)
					resolve()

			# Listen for 'Connected' signals
			wifi.on('PropertyChanged', handler)

			# # But try to read in case we registered the event handler
			# # after is was already connected
			wifi.GetPropertiesAsync()
			.then ({ Connected }) ->
				if Connected
					wifi.removeListener('PropertyChanged', handler)
					resolve()

			setTimeout ->
				wifi.removeListener('PropertyChanged', handler)
				reject()
			, timeout

getConnections = () ->
	bus.getInterfaceAsync(SERVICE, WIFI_OBJECT, TECHNOLOGY_INTERFACE)
	.then (manager) ->
		manager.ListConnectionsAsync()