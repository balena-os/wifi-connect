Promise = require 'bluebird'
DBus = require './dbus-promise'
dbus = new DBus()
bus = dbus.getBus('system')
_ = require 'lodash'

SERVICE = 'org.freedesktop.systemd1'
MANAGER_OBJECT = '/org/freedesktop/systemd1'
MANAGER_INTERFACE = 'org.freedesktop.systemd1.Manager'
UNIT_INTERFACE = 'org.freedesktop.systemd1.Unit'

exports.start = (unit, mode = 'fail') ->
	console.log('Starting ' + unit)
	bus.getInterfaceAsync(SERVICE, MANAGER_OBJECT, MANAGER_INTERFACE)
	.then (manager) ->
		manager.StartUnitAsync(unit, mode)
	.then ->
		waitUntilState(unit, 'active')

exports.stop = (unit, mode = 'fail') ->
	console.log('Stopping ' + unit)
	bus.getInterfaceAsync(SERVICE, MANAGER_OBJECT, MANAGER_INTERFACE)
	.then (manager) ->
		manager.StopUnitAsync(unit, mode)
	.then ->
		waitUntilState(unit, 'inactive')

exports.exists = (unit, mode = 'fail') ->
	bus.getInterfaceAsync(SERVICE, MANAGER_OBJECT, MANAGER_INTERFACE)
	.call('ListUnitsAsync')
	.then (units) ->
		_.has(units[0], unit)

exports.waitUntilState = waitUntilState = (unit, targetState) ->
	currentState = null
	condition = ->
		currentState != targetState
	action = ->
		getState(unit)
		.then (state) ->
			currentState = state
		.delay(1000)

	repeat(condition, action)

getState = (unit) ->
	bus.getInterfaceAsync(SERVICE, MANAGER_OBJECT, MANAGER_INTERFACE)
	.then (manager) ->
		manager.GetUnitAsync(unit)
	.then (objectPath) ->
		bus.getInterfaceAsync(SERVICE, objectPath, UNIT_INTERFACE)
	.then (unit) ->
		unit.getPropertyAsync('ActiveState')

repeat = (condition, action) ->
	Promise.try(condition)
	.then (bool) ->
		return if not bool
		Promise.try(action)
		.then ->
			repeat(condition, action)
