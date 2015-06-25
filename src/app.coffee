connman = require('connman-simplified')()
express = require('express')
app = express()
bodyParser = require('body-parser')
iptables = require('./iptables')

ssid = process.env.SSID or 'ResinAP'
passphrase = process.env.PASSPHRASE or '12345678'

port = process.env.PORT or 8080

server = null
ssidList = null

iptablesRules = [
		table: 'nat'
		chain: 'PREROUTING'
		protocol: 'tcp'
		interface: 'tether'
		dport: '80'
		jump: 'DNAT'
		target_options: 'to-destination': '0.0.0.0:8080'
	,
		table: 'nat'
		chain: 'PREROUTING'
		protocol: 'tcp'
		interface: 'tether'
		dport: '443'
		jump: 'DNAT'
		target_options: 'to-destination': '0.0.0.0:8080'
	,
		table: 'nat'
		chain: 'POSTROUTING'
		jump: 'MASQUERADE'
]


startServer = (wifi) ->
	wifi.getNetworks (err, list) ->
		throw err if err?
		ssidList = list
		wifi.openHotspot ssid, passphrase, (err) ->
			throw err if err?
			console.log("Hotspot enabled")
			iptables.appendMany iptablesRules, (err) ->
				throw err if err?
				console.log("Captive portal enabled")
				server = app.listen port, ->
					console.log("Server listening")

console.log("Starting node connman app")
connman.init (err) ->
	throw err if err?
	console.log("Connman initialized")
	connman.initWiFi (err, wifi, properties) ->
		throw err if err?
		console.log("WiFi initialized")

		app.use(bodyParser())
		app.use(express.static(__dirname + '/public'))
		app.get '/ssids', (req, res) ->
			res.send(ssidList)
		app.post '/connect', (req, res) ->
			if req.body.ssid and req.body.passphrase
				console.log("Selected " + req.body.ssid)
				res.send('OK')
				server.close ->
					iptables.deleteMany iptablesRules, (err) ->
						throw err if err?
						console.log("Server closed and captive portal disabled")
						wifi.joinWithAgent req.body.ssid, req.body.passphrase, (err) ->
							console.log(err) if err
							return startServer(wifi) if err
							console.log("Joined! Exiting.")
							process.exit()

		if !properties.connected
			console.log("Trying to join wifi")
			wifi.joinFavorite (err) ->
				if err
					startServer(wifi)
		else
			console.log("Already connected")
					

							

