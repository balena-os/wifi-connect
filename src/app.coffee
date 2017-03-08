express = require 'express'
bodyParser = require 'body-parser'

connman = require './connman'
hotspot = require './hotspot'
networkManager = require './networkManager'
systemd = require './systemd'
wifiScan = require './wifi-scan'
config = require './config'

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
		.then(run)
		.catch(error)
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

	hotspot.stop(manager, device)
	.then ->
		manager.setCredentials(req.body.ssid, req.body.passphrase)
	.then(run)
	.catch(error)

app.use (req, res) ->
	res.redirect('/')

run = ->
	manager.isSetup()
	.then (setup) ->
		if setup
			console.log('Credentials found')
			hotspot.stop(manager, device)
			.then ->
				console.log('Connecting')
				manager.connect(config.connectTimeout)
			.then ->
				console.log('Connected')
				console.log('Exiting')
				process.exit()
			.catch(error)
		else
			console.log('Credentials not found')
			hotspot.stop(manager, device)
			.then ->
				wifiScan.scanAsync()
			.then (results) ->
				ssids = results
				hotspot.start(manager, device)
			.catch(error)

app.listen(80)

retry = true
clear = true
device = null
manager = null

if process.argv[2] == '--clear=true'
	console.log('Clear is enabled')
	clear = true
else if process.argv[2] == '--clear=false'
	console.log('Clear is disabled')
	clear = false
else if not process.argv[2]?
	console.log('No clear flag passed')
	console.log('Clear is enabled')
else
	console.log('Invalid clear flag passed')
	console.log('Exiting')
	process.exit()

device = process.env.RESIN_DEVICE_TYPE
if !device
	device = process.env.DEVICE_TYPE
	if !device
		console.log('Device type not found - did you set the DEVICE_TYPE environment variable?')
		console.log('Exiting')
		process.exit()
console.log('Device type is ' + device)

systemd.exists('NetworkManager.service')
.then (result) ->
	if result
		console.log('Using NetworkManager.service')
		manager = networkManager
	else
		console.log('Using connman.service')
		manager = connman
.then ->
	if clear
		console.log('Clearing credentials')
		manager.clearCredentials()
.then ->
	manager.isSetup()
	.then (setup) ->
		if setup
			retry = false
.then ->
	manager.ready()
.then(run)
.catch (e) ->
	console.log(e)
	console.log('Exiting')
	process.exit()
