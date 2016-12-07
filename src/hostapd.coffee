Promise = require 'bluebird'
fs = Promise.promisifyAll(require('fs'))
{ spawn } = require 'child_process'

config = require './config'

ps = null

configFile = "/tmp/hostapd-#{config.iface}.conf"

exports.start = ->
	cfg =
		"""
		ssid=#{config.ssid}
		interface=#{config.iface}
		channel=6

		"""

	if config.passphrase?
		cfg +=
			"""
			wpa=2
			wpa_passphrase=#{config.passphrase}

			"""

	console.log('Starting hostapd..')
	fs.writeFileAsync(configFile, cfg)
	.then ->
		ps = spawn('hostapd', [ configFile ])
		ps.stdout.pipe(process.stdout)
		ps.stderr.pipe(process.stderr)

exports.stop = ->
	if ps is null or ps.exitCode? or ps.signalCode?
		return Promise.resolve()

	new Promise (resolve, reject) ->
		console.log('Stopping hostapd..')

		ps.kill('SIGTERM')

		timeout = setTimeout ->
			ps.kill('SIGKILL')

		ps.on 'exit', ->
			clearTimeout(timeout)
			resolve()
