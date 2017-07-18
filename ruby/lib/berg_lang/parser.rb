require_relative "output"
require_relative "parser/expression_grammar"
require_relative "parser/parse_result"
require_relative "parser/parse_state"
require_relative "parser/syntax_errors"
require_relative "parser/syntax_tree"
require_relative "parser/scanner"

module BergLang
    #
    # Parses Berg.
    #
    class Parser
        attr_reader :output
        attr_reader :base_grammar

        def initialize(output: Output.new(STDOUT))
            @output = output
            @base_grammar = ExpressionGrammar.new(output)
        end

        #
        # Parses all tokens from the source.
        #
        def parse(source)
            parse_result = ParseResult.new(source, output)
            stream = source.open
            state = ParseState.new(parse_result, base_grammar.scanner(stream))

            # Read each token from the input
            state.advance(stream.index, state.scanner.grammar.sof, stream.index, state.scanner.next_is_space?)
            while state.scan_next
            end
            state.advance(stream.index, state.scanner.grammar.eof, stream.index, false)

            parse_result
        end

        private

        include SyntaxErrors
    end
end
