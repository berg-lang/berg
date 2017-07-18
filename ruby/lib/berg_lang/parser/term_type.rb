module BergLang
    class Parser
        class TermType
            attr_reader :grammar
            attr_reader :name
            attr_reader :token_name
            attr_reader :string
            attr_reader :left
            attr_reader :right
            attr_reader :priority
            attr_reader :space
            attr_reader :significant
            attr_reader :indented_variant
            attr_reader :statement_boundary
            attr_reader :next_grammar

            def initialize(grammar, name, token_name: name, string: nil, left: nil, right: nil, space: false, significant: true, indented_variant: nil, statement_boundary: !!indented_variant, next_grammar: nil)
                @grammar = grammar

                left = Side.new(**left) if left
                right = Side.new(**right) if right
                @name = name
                @token_name = token_name
                @string = string
                @left = left
                @right = right
                @space = space
                @significant = significant
                @indented_variant = indented_variant
                @statement_boundary = statement_boundary
                @next_grammar = next_grammar
                if !significant?
                    @priority = INSIGNIFICANT_PRIORITY
                elsif !left == !right
                    @priority = EXPRESSION_INFIX_PRIORITY
                else
                    @priority = PREFIX_POSTFIX_PRIORITY
                end
            end

            def space?
                !!space
            end

            def significant?
                significant
            end

            def output
                grammar.output
            end

            def to_s
                name.is_a?(String) ? name : name.inspect
            end

            def fixity
                if left
                    return right ? :infix : :postfix
                else
                    return right ? :prefix : :expression
                end
            end

            def expression?
                !left && !right
            end
            def infix?
                left && right
            end
            def prefix?
                !left && right
            end
            def postfix?
                left && !right
            end

            def left_is_operand?
                !left
            end
            def right_is_operand?
                !right
            end

            def left_accepts_operand_type?(operand_type)
                operand_type.right.nil? || left.accepts_operand_type?(operand_type)
            end
            def right_accepts_operand_type?(operand_type)
                operand_type.left.nil? || right.accepts_operand_type?(operand_type)
            end

            private

            PREFIX_POSTFIX_PRIORITY = 1
            EXPRESSION_INFIX_PRIORITY = 2
            INSIGNIFICANT_PRIORITY = 3

            #
            # Represents the left or right side of a term type.
            #
            # This lets you find out what precedence and associativity a term has, as well as
            # whether it *needs* an expression on that side (like + or ? or !) or *wants* an
            # expression on that side.
            #
            class Side
                attr_accessor :accepts_operands
                attr_accessor :declaration
                attr_accessor :opened_by
                attr_accessor :closed_by

                def initialize(accepts_operands: Set.new, declaration: nil, opened_by: nil, closed_by: nil)
                    @accepts_operands = accepts_operands
                    @declaration = declaration
                    @opened_by = opened_by
                    @closed_by = closed_by
                end

                def accepts_operand_type?(term_type)
                    return term_type != opened_by if opened_by
                    return term_type != closed_by if closed_by
                    accepts_operands.include?(term_type)
                end

                def declaration?
                    @declaration
                end
            end
        end
    end
end
