module BergLang
    class Parser
        class SyntaxError < StandardError
            attr_reader :name
            attr_reader :ast
            attr_reader :args
            attr_reader :error
            attr_reader :remedy

            def initialize(name, ast:, args:, error:, remedy:)
                @name = name
                @ast = ast
                @args = args
                @error = error
                @remedy = remedy
                line = source_range.begin_location[0]
                super("#{ast.source_range.source.name}:#{line}: #{error} #{remedy}")
            end

            def source_range
                ast.source_range
            end
        end
    end
end
