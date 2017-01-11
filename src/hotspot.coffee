Promise = require 'bluebird'
{ spawn, exec } = require 'child_process'
execAsync = Promise.promisify(exec)

config = require './config'
hostapd = require './hostapd'
dnsmasq = require './dnsmasq'
modprobe = require './modprobe'

started = false

exports.start = (manager, device) ->
	if started
		return Promise.resolve()

	started = true

	console.log('Starting hotspot')

	modprobe.hotspot(device)
	.then ->
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
	.then ->
		console.log('Started hotspot')

exports.stop = (manager, device) ->
	if not started
		return Promise.resolve()

	started = false

	console.log('Stopping hotspot')

	modprobe.normal(device)
	.then ->
		hostapd.stop()
	.then ->
		dnsmasq.stop()
	.then ->
		manager.start()
	.then ->
		console.log('Stopped hotspot')
