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

error = (e) ->
	console.log(e)
	if retry
		console.log('Retrying')
		console.log('Clearing credentials')
		manager.clearCredentials()
		.then ->
			run()
		.catch (e) ->
			error(e)
	else
		console.log('Not retrying')
		console.log('Exiting')
		process.exit()

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
	.catch (e) ->
		error(e)

app.use (req, res) ->
	res.redirect('/')

run = ->
	manager.isSetup()
	.then (setup) ->
		if setup
			console.log('Credentials found')
			hotspot.stop()
			.then ->
				console.log('Connecting')
				manager.connect(15000)
			.then ->
				console.log('Connected')
				console.log('Exiting')
				process.exit()
			.catch (e) ->
				error(e)
		else
			console.log('Credentials not found')
			wifiScan.scanAsync()
			.then (results) ->
				ssids = results
				hotspot.start(manager)
			.catch (e) ->
				error(e)

app.listen(80)

retry = false
if process.argv[2] == '--retry=true'
	console.log('Retry enabled')
	retry = true
else if process.argv[2] == '--retry=false'
	console.log('Retry disabled')
else if not process.argv[2]?
	console.log('No retry flag passed')
	console.log('Retry disabled')
else
	console.log('Invalid retry flag passed')
	console.log('Exiting')
	process.exit()

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
	run()
.catch (e) ->
	console.log(e)
	console.log('Exiting')
	process.exit()
