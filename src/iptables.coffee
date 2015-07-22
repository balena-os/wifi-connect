async = require('async')
exec = require('child_process').exec

iptables = {}

iptables.append = (rule, cb) ->
	exec("iptables -t #{rule.table} -A #{rule.rule}", cb)

iptables.delete = (rule, cb) ->
	exec("iptables -t #{rule.table} -D #{rule.rule}", cb)

iptables.createChain = (table, chain, cb) ->
	exec("iptables -t #{table} -N #{chain}", cb)

iptables.flush = (table, chain, cb) ->
	exec("iptables -t #{table} -F #{chain}", cb)

iptables.appendMany = (rules, cb) ->
	async.eachSeries rules, iptables.append, cb

iptables.deleteMany = (rules, cb) ->
	async.eachSeries rules, iptables.delete, cb

module.exports = iptables
