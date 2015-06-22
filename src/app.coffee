connman = require('connman-simplified')()
app = require('express')()
bodyParser = require('body-parser')

ssid = process.env.SSID or 'ResinAP'
passphrase = process.env.PASSPHRASE or '12345678'

port = process.env.PORT or 8080

server = null
ssidList = null


startServer = (wifi) ->
	wifi.getNetworks (err, list) ->
		ssidList = list
		wifi.openHotspot ssid, passphrase, (err) ->
			throw err if err
			console.log("Hotspot enabled")
			server = app.listen(port)

console.log("Starting node connman app")
connman.init (err) ->
	throw err if err
	console.log("Connman initialized")
	connman.initWiFi (err, wifi, properties) ->
		throw err if err
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
				wifi.join req.body.ssid, req.body.passphrase, (err) ->
					startServer(wifi) if err

		if !properties.connected
			wifi.joinFavorite (err) ->
				if err
					startServer(wifi)
					

							

