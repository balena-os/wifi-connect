Promise = require 'bluebird'
fs = Promise.promisifyAll(require('fs'))
{ spawn } = require 'child_process'

config = require './config'

ps = null

configFile = "/tmp/dnsmasq-#{config.iface}.conf"

exports.start = ->
	cfg =
		"""
		interface=#{config.iface}
		address=/#/#{config.gateway}
		dhcp-range=#{config.dhcpRange}
		bind-interfaces

		"""

	console.log('Starting dnsmasq..')
	fs.writeFileAsync(configFile, cfg)
	.then ->
		ps = spawn('dnsmasq', [ '--keep-in-foreground', '-C', configFile ])
		ps.stdout.pipe(process.stdout)
		ps.stderr.pipe(process.stderr)

exports.stop = ->
	if ps is null or ps.exitCode? or ps.signalCode?
		return Promise.resolve()

	new Promise (resolve, reject) ->
		console.log('Stopping dnsmasq..')

		ps.kill('SIGTERM')

		timeout = setTimeout ->
			ps.kill('SIGKILL')

		ps.on 'exit', ->
			clearTimeout(timeout)
			resolve()
