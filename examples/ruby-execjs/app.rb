require "execjs"

value = ExecJS.eval "'hello from execjs'.toUpperCase()"
puts value
