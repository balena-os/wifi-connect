Promise = require 'bluebird'
DBus = require './dbus-promise'

dbus = new DBus()

bus = dbus.getBus('system')

config = require './config'
fs = Promise.promisifyAll(require('fs'))
systemd = require './systemd'
utils = require './utils'

SERVICE = 'net.connman'
WIFI_OBJECT = '/net/connman/technology/wifi'
TECHNOLOGY_INTERFACE = 'net.connman.Technology'

exports.start = ->
	systemd.start('connman.service')

exports.stop = ->
	systemd.stop('connman.service')

exports.isSetup = ->
	new Promise (resolve, reject) ->
		utils.exists(config.persistentConfig)
		.then (exists) ->
			if exists
				utils.copyFile(config.persistentConfig, config.connmanConfig)
				.then ->
					resolve (true)
				.catch (e) ->
					reject (e)
			else
				resolve (false)

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
	fs.unlinkAsync(config.persistentConfig)

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
