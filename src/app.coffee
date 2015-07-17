Promise = require('bluebird')
connman = Promise.promisifyAll(require('connman-simplified')())
express = require('express')
app = express()
bodyParser = require('body-parser')
iptables = Promise.promisifyAll(require('./iptables'))
spawn = require('child_process').spawn
os = require('os')
async = require('async')

ssid = process.env.SSID or 'ResinAP'
passphrase = process.env.PASSPHRASE or null

port = process.env.PORT or 8080

server = null
ssidList = null
dnsServer = null

getIptablesRules = (callback) ->
	async.retry {times: 100, interval: 100}, (cb) ->
		Promise.try ->
			os.networkInterfaces().tether[0].address
		.nodeify(cb)
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
	console.log("Getting networks list")
	wifi.getNetworksAsync().then (list) ->
		ssidList = list
	.then ->
		wifi.openHotspotAsync(ssid, passphrase)
	.then ->
			console.log("Hotspot enabled")
			dnsServer = spawn('named', ['-f'])
			getIptablesRules (err, iptablesRules) ->
				iptables.appendManyAsync(iptablesRules)
				.then ->
					console.log("Captive portal enabled")
					server = app.listen port, ->
						console.log("Server listening")

console.log("Starting node connman app")
manageConnection = (retryCallback) ->
	connman.initAsync()
	.then ->
		console.log("Connman initialized")
		connman.initWiFiAsync()
	.spread (wifi, properties) ->
		wifi = Promise.promisifyAll(wifi)
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
						wifi.joinWithAgentAsync(req.body.ssid, req.body.passphrase)
						.then ->
							console.log("Joined! Exiting.")
							retryCallback()
						.catch (err) ->
							console.log(err)
							return startServer(wifi)
		app.use (req, res) ->
			res.redirect('/')

		# Create TETHER iptables chain (will silently fail if it already exists)
		iptables.createChainAsync('nat', 'TETHER')
		.catch ->
			return
		.then ->
			iptables.deleteAsync({ table: 'nat', rule: "PREROUTING -i tether -j TETHER"})
		.catch ->
			return
		.then ->
				iptables.flushAsync('nat', 'TETHER')
		.then ->
			if !properties.connected
				console.log("Trying to join wifi")
				wifi.joinFavoriteAsync()
				.catch (err) ->
					startServer(wifi)
			else
				console.log("Already connected")
				retryCallback()
	.catch (err) ->
		console.log(err)
		return retryCallback(err)

async.retry {times: 10, interval: 1000}, manageConnection, (err) ->
	throw err if err?
	process.exit()
							

