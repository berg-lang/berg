module BergLang
    class Parser
        class SyntaxError < StandardError
            attr_reader :name
            attr_reader :ast
            attr_reader :args
            attr_reader :error
            attr_reader :remedy
            attr_reader :source

            def initialize(name, source:, ast:, args:, error:, remedy:)
                @name = name
                @source = source
                @ast = ast
                @args = args
                @error = error
                @remedy = remedy
                line, column = source.location(ast.source_range.end)
                super("#{source.name}:#{line}: #{error} #{remedy}")
            end
        end
    end
end
