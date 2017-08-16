require_relative "berg_lang/parser"
require_relative "berg_lang/version"

module BergLang
    def self.parse_file(filename, output=default_output)
        parser = Parser.new(Source::FileSource.new(filename), output)
        parser.parse
        [ parser.syntax_tree_builder, syntax_tree, ]
    end

    def self.parse_string(name, string, output=default_output)
        Parser.new(Source::StringSource.new(name, string), output).parse
    end

    def self.default_output
        @default_output ||= Output.new(STDOUT)
    end

    def self.grammar
        @grammar ||= Parser::Grammar::LineStartGrammar.new
    end
end
