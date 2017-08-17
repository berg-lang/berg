module BergLang
    module Parser
        class TokenType
            #
            # The name of this token for printing.
            #
            attr_reader :name

            #
            # The operation type of this token.
            #
            # @return [:expression, :infix, :prefix, :postfix] The operation type.
            #
            attr_reader :operation_type

            #
            # Whether this operator opens a block--for example, `:` or `extend`.
            #
            def block_operator?
                @block_operator
            end

            #
            # The grammar to use after this token is read.
            #
            attr_reader :next_grammar

            #
            # The next operator to expect (for things like `()` and `?:`).
            #
            attr_reader :next_operator

            #
            # The previous operator in the chain (for things like `()` and `?:`).
            #
            attr_reader :prev_operator

            #
            # Create a new token type.
            #
            def initialize(name, operation_type, block_operator: false, next_grammar: nil, next_operator: nil, precedence: nil)
                raise "Invalid operation type!" unless [:expression,:infix,:prefix,:postfix].include?(operation_type)
                @name = name
                @operation_type = operation_type
                @block_operator = block_operator
                @next_grammar = next_grammar
                @next_operator = next_operator
                @prev_operator = []
                next_operator.prev_operator << self if next_operator
            end
        end
    end
end
