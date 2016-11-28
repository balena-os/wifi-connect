Promise = require 'bluebird'
{ spawn, exec } = require 'child_process'
execAsync = Promise.promisify(exec)

config = require './config'
hostapd = require './hostapd'
dnsmasq = require './dnsmasq'

started = false

exports.start = (manager) ->
	if started
		return Promise.resolve()

	started = true

	console.log('Stopping service, starting hotspot')

	manager.stop()
	.then ->
		execAsync('rfkill unblock wifi')
	.then ->
		# XXX: detect if the IP is already set instead of doing `|| true`
		execAsync("ip addr add #{config.gateway}/24 dev #{config.iface} || true")
	.then ->
		hostapd.start()
	.then ->
		dnsmasq.start()

exports.stop = (manager) ->
	if not started
		return Promise.resolve()

	console.log('Starting service, stopping hotspot')

	started = false

	Promise.all [
		hostapd.stop()
		dnsmasq.stop()
	]
	.then ->
		manager.start()
