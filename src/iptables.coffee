async = require('async')
iptables = require('netfilter').iptables

iptables.appendMany = (rules, cb) ->
	async.eachSeries rules, iptables.append, cb

iptables.deleteMany = (rules, cb) ->
	async.eachSeries rules, iptables.delete, cb

module.exports = iptables
