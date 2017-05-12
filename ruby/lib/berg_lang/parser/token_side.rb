require "set"

module BergLang
    class Parser
        #
        # Represents the left or right side of a token type.
        #
        # This lets you find out what precedence and associativity a token has, as well as
        # whether it *needs* an expression on that side (like + or ? or !) or *wants* an
        # expression on that side.
        #
        class TokenSide
            attr_reader :opens_indent_block
            attr_reader :can_be_sticky
            attr_reader :resolve_manually

            def initialize(is_operator:, accepts_children: Set.new, opens_indent_block: nil, can_be_sticky: nil, resolve_manually: nil)
                @is_operator = is_operator
                @accepts_children = accepts_children
                @opens_indent_block = opens_indent_block
                @can_be_sticky = can_be_sticky
                @resolve_manually = resolve_manually
            end

            def needs_expression?
                @is_operator
            end

            def needs_operator?
                !@is_operator
            end

            def operator?
                @is_operator
            end

            def expression?
                !@is_operator
            end

            def can_have_child?(token_type)
                accepts_children.include?(token_type)
            end

            def accepts_children
                @accepts_children
            end
        end
    end
end