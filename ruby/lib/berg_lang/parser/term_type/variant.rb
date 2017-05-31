require_relative "../term_type"

module BergLang
    class Parser
        class TermType
            class Variant < TermType
                attr_reader :string
                attr_reader :left
                attr_reader :right
                attr_reader :priority

                def initialize(name, string: name, left: nil, right: nil, filler: nil)
                    super(name)
                    raise "opens_indent_block unsupported on the left side" if left && left.opens_indent_block?
                    @string = string
                    @left = left
                    @right = right
                    @filler = filler
                    if !left == !right
                        raise "Filler term #{name} cannot be #{fixity} (must be prefix or postfix)" if filler?
                        @priority = EXPRESSION_INFIX_PRIORITY
                    elsif filler?
                        @priority = FILLER_PRIORITY
                    else
                        @priority = PREFIX_POSTFIX_PRIORITY
                    end
                end

                def filler?
                    @filler
                end

                def fixity
                    if left
                        return right ? :infix : :postfix
                    else
                        return right ? :prefix : :expression
                    end
                end

                def expression
                    self if !left && !right
                end
                def infix
                    self if left && right
                end
                def prefix
                    self if !left && right
                end
                def postfix
                    self if left && !right
                end

                def left_is_operand?
                    !left
                end
                def right_is_operand?
                    !right
                end

                def variants
                    return enum_for(:variants) unless block_given?
                    yield self
                end

                private

                PREFIX_POSTFIX_PRIORITY = 1
                EXPRESSION_INFIX_PRIORITY = 2
                FILLER_PRIORITY = 3
            end
        end
    end
end
