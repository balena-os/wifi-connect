Promise = require 'bluebird'
DBus = require 'dbus'
Bus = require 'dbus/lib/bus'
Interface = require 'dbus/lib/interface'

Promise.promisifyAll(Bus.prototype)
Promise.promisifyAll(Interface.prototype)

oldInit = Interface::init

Interface::init = (args...) ->
	oldInit.apply(this, args)

	for own method of @object.method
		this[method + 'Async'] = do (method) -> (args...) ->
			new Promise (resolve, reject) =>
				this[method].timeout = 5000
				this[method].finish = resolve
				this[method].error = reject
				this[method](args...)

module.exports = DBus
