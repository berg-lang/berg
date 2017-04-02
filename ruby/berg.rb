require_relative "berg/source"
require_relative "berg/parser"
require "pp"

str = ARGV[0]
source = Berg::Source.new("<argument>", str)
parser = Berg::Parser.new(source)
expression = parser.parse
puts "\n"
puts str
puts "---"
puts expression