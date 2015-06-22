connman = require 'connman-simplified'
app = require('express')()
bodyParser = require('body-parser')

console.log("Starting node connman app")
connman.init (err) ->
	throw err if err
	console.log("Connman initialized")
	connman.initWiFi (err, wifi, properties) ->
		throw err if err
		console.log("WiFi initialized")
		if !properties.connected
			wifi.joinFavorite (err) ->
				if err
					wifi.openHotspot 'raspberrypi', (err) ->
						throw err if err
						console.log("Hotspot enabled")

						app.use(bodyParser())
						app.use(express.static(__dirname + '/public'))
						app.get '/ssids', (req, res) ->
							res.send(['net1', 'net2'])
						app.post '/connect', (req, res) ->
							if req.body.ssid and req.body.passphrase
								console.log("Selected " + req.body.ssid)

