Promise = require 'bluebird'
{ spawn, exec } = require 'child_process'
execAsync = Promise.promisify(exec)

exports.hotspot = (device) ->
	switch device
		when 'intel-edison', 'edison'
			run('modprobe -r bcm4334x', 'modprobe bcm4334x op_mode=2 firmware_path="firmware/intel-edison/fw_bcmdhd.bin" nvram_path="firmware/intel-edison/bcmdhd.cal"')
		else return Promise.resolve()

exports.normal = (device) ->
	switch device
		when 'intel-edison', 'edison'
			run('modprobe -r bcm4334x', 'modprobe bcm4334x firmware_path="firmware/intel-edison/fw_bcmdhd.bin" nvram_path="firmware/intel-edison/bcmdhd.cal"')
		else return Promise.resolve()

run = (disable, enable) ->
	console.log('Loading kernel module: ' + enable)
	execAsync(disable)
	.delay(1000)
	.then ->
		execAsync(enable)
	.delay(1000)
