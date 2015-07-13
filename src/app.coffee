connman = require('connman-simplified')()
express = require('express')
app = express()
bodyParser = require('body-parser')
iptables = require('./iptables')
spawn = require('child_process').spawn
os = require('os')
async = require('async')

ssid = process.env.SSID or 'ResinAP'
passphrase = process.env.PASSPHRASE or '12345678'
passphrase = null if process.env.UNSECURED_TETHERING?

port = process.env.PORT or 8080

server = null
ssidList = null
dnsServer = null

getIptablesRules = (callback) ->
	async.retry {times: 100, interval: 100}, (cb) ->
		try
			myIP = os.networkInterfaces().tether[0].address
			cb(null, myIP)
		catch err
			cb(err)
	, (err, myIP) ->
		throw err if err?
		callback null, [
				table: 'nat'
				rule: "PREROUTING -i tether -j TETHER"
			,
				table: 'nat'
				rule: "TETHER -p tcp --dport 80 -j DNAT --to-destination #{myIP}:8080"
			,
				table: 'nat'
				rule: "TETHER -p tcp --dport 443 -j DNAT --to-destination #{myIP}:8080"
			,
				table: 'nat'
				rule: "TETHER -p udp --dport 53 -j DNAT --to-destination #{myIP}:53"
		]


startServer = (wifi) ->
	wifi.getNetworks (err, list) ->
		throw err if err?
		ssidList = list
		wifi.openHotspot ssid, passphrase, (err) ->
			throw err if err?
			console.log("Hotspot enabled")
			dnsServer = spawn('named', ['-f'])
			getIptablesRules (err, iptablesRules) ->
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
				server.close()
				iptables.delete { table: 'nat', rule: "PREROUTING -i tether -j TETHER"}, ->
					iptables.flush 'nat', 'TETHER', ->
						dnsServer.kill()
						console.log("Server closed and captive portal disabled")
						wifi.joinWithAgent req.body.ssid, req.body.passphrase, (err) ->
							console.log(err) if err
							return startServer(wifi) if err
							console.log("Joined! Exiting.")
							process.exit()
		app.use (req, res) ->
			res.redirect('/')

		# Create TETHER iptables chain (will silently fail if it already exists)
		iptables.createChain 'nat', 'TETHER', ->
			# Ensure no rules exist from an unclean shutdown
			iptables.delete { table: 'nat', rule: "PREROUTING -i tether -j TETHER"}, ->
				iptables.flush 'nat', 'TETHER', ->
					if !properties.connected
						console.log("Trying to join wifi")
						wifi.joinFavorite (err) ->
							if err
								startServer(wifi)
					else
						console.log("Already connected")
						process.exit()
						

							

