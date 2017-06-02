module BergLang
    class Parser
        class TermType
            attr_reader :name
            attr_reader :token_name
            attr_reader :string
            attr_reader :left
            attr_reader :right
            attr_reader :priority

            def initialize(name, token_name: name, string: nil, left: nil, right: nil, space: nil)
                left = Side.new(**left) if left
                right = Side.new(**right) if right
                raise "opens_indent_block unsupported on the left side" if left && left.opens_indent_block?
                @name = name
                @token_name = token_name
                @string = string
                @left = left
                @right = right
                @space = space
                if !left == !right
                    raise "space term cannot be #{fixity} (must be prefix or postfix)" if space?
                    @priority = EXPRESSION_INFIX_PRIORITY
                elsif space?
                    @priority = SPACE_PRIORITY
                else
                    @priority = PREFIX_POSTFIX_PRIORITY
                end
            end

            def to_s
                name.to_s
            end

            def space?
                @space
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

            def left_accepts_operand?(type)
                type.right.nil? || left.accepts_operand?(type)
            end
            def right_accepts_operand?(type)
                type.left.nil? || right.accepts_operand?(type)
            end

            private

            PREFIX_POSTFIX_PRIORITY = 1
            EXPRESSION_INFIX_PRIORITY = 2
            SPACE_PRIORITY = 3

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
