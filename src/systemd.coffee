Promise = require('bluebird')
execAsync = Promise.promisify(require('child_process').exec)

exports.callMethod = callMethod = (method, methodArgs) ->
	execAsync("dbus-send --print-reply --reply-timeout=2000 --type=method_call --system --dest=org.freedesktop.systemd1 /org/freedesktop/systemd1 org.freedesktop.systemd1.Manager.#{method} #{methodArgs}")	

exports.reboot = reboot = ->
	execAsync('sync')
	.then ->
		callMethod('Reboot')
	.then ->
		process.exit()

exports.stop = stop = (service) ->
	callMethod('StopUnit', "string:\"#{service}.service\" string:\"fail\"")

exports.start = start = (service) ->
	callMethod('StartUnit', "string:\"#{service}.service\" string:\"fail\"")

exports.restart = restart = (service) ->
	callMethod('RestartUnit', "string:\"#{service}.service\" string:\"fail\"")