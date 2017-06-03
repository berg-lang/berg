require_relative "output"
require_relative "parser/expression_grammar"
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
            stream = source.open
            state = ParseState.new(source, base_grammar.scanner(stream))

            # Read each token from the input
            state.advance(stream.index,state.scanner.grammar.sof, stream.index, state.scanner.next_is_space?)
            while state.scan_next
            end
            state.advance(stream.index, state.scanner.grammar.eof, stream.index, false)

            state.syntax_tree
        end

        def resolve(scanner, state, token_start, token_end, type)
            next_grammar = type.resolve(state, token_start, token_end, scanner.next_is_space?)
            if next_grammar && next_grammar != scanner.grammar
                scanner = next_grammar.scanner(stream)
            end
            scanner
        end

        private

        include SyntaxErrors
    end
end
