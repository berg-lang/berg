module BergLang
    module Parser
        class TokenType
            # The name of this token for printing.
            attr_reader :name
            # The operation_type of this token (`:expression`, `:binary`, `:prefix`, `:suffix`).
            attr_reader :operation_type
            # The symbol that this token lives under.
            attr_accessor :symbol_type
            # Whether this operator requires a child block (even if empty), a la `:`.
            attr_reader :expects_child_block
            # The grammar to use after this token is read.
            attr_reader :next_grammar
            # The grammar to return to after this token is closed.
            attr_reader :close_grammar
            # The tokens that can open this (if this is a close token).
            attr_reader :opened_by
            # The token that closes this (if this is an open token).
            attr_reader :closed_by

            def initialize(name, operation_type, expects_child_block: false, next_grammar: nil, close_grammar: nil, closed_by: nil)
                @name = name
                @operation_type = operation_type
                @expects_child_block = expects_child_block
                @next_grammar = next_grammar
                @close_grammar = close_grammar
                @closed_by = closed_by
                @opened_by = []
                closed_by.opened_by << self
            end
        end
    end
end
