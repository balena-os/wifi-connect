Promise = require 'bluebird'
DBus = require './dbus-promise'

dbus = new DBus()

bus = dbus.getBus('system')

SERVICE = 'net.connman'
WIFI_OBJECT = '/net/connman/technology/wifi'
TECHNOLOGY_INTERFACE = 'net.connman.Technology'

exports.waitForConnection = (timeout) ->
	console.log('Waiting for connman to connect..')

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
