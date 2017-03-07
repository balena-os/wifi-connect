Promise = require 'bluebird'
DBus = require './dbus-promise'
fs = Promise.promisifyAll(require('fs'))
dbus = new DBus()
bus = dbus.getBus('system')
_ = require 'lodash'

config = require './config'
systemd = require './systemd'
utils = require './utils'

SERVICE = 'net.connman'
WIFI_OBJECT = '/net/connman/technology/wifi'
TECHNOLOGY_INTERFACE = 'net.connman.Technology'

exports.start = ->
	systemd.start('connman.service')

exports.stop = ->
	systemd.stop('connman.service')

exports.ready = ->
	systemd.waitUntilState('connman.service', 'active')

exports.isSetup = ->
	fs.statAsync(config.persistentConfig)
	.then ->
		utils.copyFile(config.persistentConfig, config.connmanConfig)
	.return(true)
	.catchReturn(false)

exports.setCredentials = (ssid, passphrase) ->
	connection = """
		[service_home_ethernet]
		Type = ethernet
		Nameservers = 8.8.8.8,8.8.4.4

		[service_home_wifi]
		Type = wifi
		Name = #{ssid}
		Passphrase = #{passphrase}
		Nameservers = 8.8.8.8,8.8.4.4

	"""

	console.log('Saving connection')
	console.log(connection)

	utils.durableWriteFile(config.persistentConfig, connection)

exports.clearCredentials = ->
	fs.unlinkAsync(config.persistentConfig)
	.catch(code: 'ENOENT', _.noop)

exports.connect  = (timeout) ->
	bus.getInterfaceAsync(SERVICE, WIFI_OBJECT, TECHNOLOGY_INTERFACE)
	.then (manager) ->
		new Promise (resolve, reject) ->
			handler = (name, value) ->
				if name is 'Connected' and value is true
					manager.removeListener('PropertyChanged', handler)
					resolve()

			# Listen for 'Connected' signals
			manager.on('PropertyChanged', handler)

			# But try to read in case we registered the event handler
			# after is was already connected
			manager.GetPropertiesAsync()
			.then ({ Connected }) ->
				if Connected
					manager.removeListener('PropertyChanged', handler)
					resolve()

			setTimeout ->
				manager.removeListener('PropertyChanged', handler)
				reject(new Error('Timed out'))
			, timeout
