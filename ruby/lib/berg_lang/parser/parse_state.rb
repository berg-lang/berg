module BergLang
    module Parser
        class ParseState
            attr_reader :output
            attr_reader :stream
            attr_reader :parser
            attr_reader :resolver
            attr_reader :syntax_tree_builder

            attr_reader :last_symbol_scanned
            attr_reader :last_token_resolved
            attr_reader :last_token_complete

            attr_reader :source_data
            attr_reader :output
            attr_reader :last_symbol_type
            attr_reader :last_symbol_start

            def initialize(stream, grammar, output)
                source_data = SourceData.new(stream.source)
                @scanner = Scanner.new(stream, grammar, source_data, output)
                syntax_tree_builder = SyntaxTreeBuilder.new(source_data, output)
                @resolver = Resolver.new(syntax_tree_builder, output)
            end

        end
    end
end
