connman = require 'connman-simplified'

console.log("Starting node connman app")
connman.init (err) ->
	throw err if err
	console.log("Connman initialized")
	connman.initWiFi (err, wifi, properties) ->
		throw err if err
		console.log("WiFi initialized")
		wifi.openHotspot 'raspi', (err) ->
			throw err if err
			console.log("Hotspot enabled")