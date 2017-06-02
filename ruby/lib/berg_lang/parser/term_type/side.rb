require "set"

module BergLang
    class Parser
        class TermType
            #
            # Represents the left or right side of a term type.
            #
            # This lets you find out what precedence and associativity a term has, as well as
            # whether it *needs* an expression on that side (like + or ? or !) or *wants* an
            # expression on that side.
            #
            class Side
                attr_accessor :accepts_operands
                attr_reader :opens_indent_block
                attr_reader :declaration
                attr_accessor :opened_by
                attr_accessor :closed_by

                def initialize(accepts_operands: Set.new, opens_indent_block: nil, declaration: false, opened_by: nil, closed_by: nil)
                    @accepts_operands = accepts_operands
                    @opens_indent_block = opens_indent_block
                    @declaration = declaration
                    @opened_by = opened_by
                    @closed_by = closed_by
                end

                def accepts_operand?(term_type)
                    return term_type != opened_by if opened_by
                    return term_type != closed_by if closed_by
                    accepts_operands.include?(term_type)
                end

                def opens_indent_block?
                    @opens_indent_block
                end

                def declaration?
                    @declaration
                end
            end
        end
    end
end