module BergLang
    class Parser
        class TermType
            class Variant
                attr_reader :name
                attr_reader :string
                attr_reader :left
                attr_reader :right
                attr_reader :priority

                def initialize(name, string: nil, left: nil, right: nil, space: nil)
                    raise "opens_indent_block unsupported on the left side" if left && left.opens_indent_block?
                    @name = name
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
            end
        end
    end
end
