Promise = require 'bluebird'
fs = Promise.promisifyAll(require('fs'))
constants = require 'constants'
path = require 'path'

exports.durableWriteFile = (file, data) ->
	fs.writeFileAsync(file + '.tmp', data)
	.then ->
		fs.openAsync(file + '.tmp', 'r')
	.tap(fs.fsyncAsync)
	.then(fs.closeAsync)
	.then ->
		fs.renameAsync(file + '.tmp', file)
	.then ->
		fs.openAsync(path.dirname(file), 'r', constants.O_DIRECTORY)
	.tap(fs.fsyncAsync)
	.then(fs.closeAsync)

exports.copyFile = (source, target) ->
	fs.readFileAsync(source)
	.then (rf) ->
		fs.writeFileAsync(target, rf)
