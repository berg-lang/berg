require_relative "output"
require_relative "parser/expression_grammar"
require_relative "parser/state"
require_relative "parser/syntax_errors"
require_relative "parser/syntax_tree"
require_relative "parser/scanner"

module BergLang
    #
    # Parses Berg.
    #
    # Disambiguation rules:
    # 1. The earliest thing we can insert, wins.
    # 2. infix>postfix and expression>prefix if either one works fine.
    class Parser
        attr_reader :source
        attr_reader :output
        attr_reader :grammar

        def initialize(source, output: Output.new(STDOUT))
            @source = source
            @output = output
            @grammar = ExpressionGrammar.new(output)
        end

        #
        # Parses all tokens from the source.
        #
        def parse
            state = State.new(self)
            scanner = Scanner.new(self)

            # Read each token from the input
            term_start = scanner.index
            while type = scanner.next
                term_end = scanner.index
                type.resolve(state, term_start, term_end, scanner.next_is_space?)
                term_start = term_end
            end

            state.syntax_tree
        end

        private

        include SyntaxErrors
    end
end
