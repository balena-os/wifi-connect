express = require 'express'
bodyParser = require 'body-parser'

connman = require './connman'
hotspot = require './hotspot'
networkManager = require './networkManager'
systemd = require './systemd'
wifiScan = require './wifi-scan'

app = express()

app.use(bodyParser.json())
app.use(bodyParser.urlencoded(extended: true))
app.use(express.static(__dirname + '/public'))

ssids = []

app.get '/ssids', (req, res) ->
	res.json(ssids)

app.post '/connect', (req, res) ->
	if not (req.body.ssid? and req.body.passphrase?)
		return res.sendStatus(400)

	res.send('OK')

	hotspot.stop(manager)
	.then ->
		manager.setCredentials(req.body.ssid, req.body.passphrase)
	.then ->
		run()

app.use (req, res) ->
	res.redirect('/')

run = ->
	manager.isSetup()
	.then (isSetup) ->
		if isSetup
			console.log('Credentials found')
			hotspot.stop()
			.catch (e) ->
				console.log(e)
				console.log('Exiting')
				process.exit()
			.then ->
				console.log('Connecting')
				manager.connect(15000)
			.then ->
				console.log('Connected')
				console.log('Exiting')
				process.exit()
			.catch (e) ->
				if retry
					console.log('Clearing credentials')
					manager.clearCredentials()
					.then ->
						console.log("here")
						run()
					.catch (e) ->
						console.log(e)
						console.log('Exiting')
						process.exit()
				else
					run()
		else
			console.log('Credentials not found')
			hotspot.start(manager)
			.catch (e) ->
				console.log(e)
				console.log('Exiting')
				process.exit()

app.listen(80)

retry = null
if process.argv[2] == 'retry'
	console.log("Retry enabled")
	retry = true
else
	console.log("Retry disabled")
	retry = false

manager = null
systemd.exists('NetworkManager.service')
.then (result) ->
	if result
		console.log('Using NetworkManager.service')
		manager = networkManager
	else
		console.log('Using connman.service')
		manager = connman
.then ->
	wifiScan.scanAsync()
.then (results) ->
	ssids = results
.then ->
	run()
.catch (e) ->
	console.log(e)
	console.log('Exiting')
	process.exit()
