DBus = require './dbus-promise'

dbus = new DBus()

bus = dbus.getBus('system')

SERVICE = 'org.freedesktop.systemd1'
MANAGER_OBJECT = '/org/freedesktop/systemd1'
MANAGER_INTERFACE = 'org.freedesktop.systemd1.Manager'

exports.start = (unit, mode='fail') ->
	bus.getInterfaceAsync(SERVICE, MANAGER_OBJECT, MANAGER_INTERFACE)
	.then (manager) ->
		manager.StartUnitAsync(unit, mode)

exports.stop = (unit, mode='fail') ->
	bus.getInterfaceAsync(SERVICE, MANAGER_OBJECT, MANAGER_INTERFACE)
	.then (manager) ->
		manager.StopUnitAsync(unit, mode)
