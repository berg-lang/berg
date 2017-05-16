require_relative "../term_type"

module BergLang
    class Parser
        class TermType
            class Definite < TermType
                attr_reader :string
                attr_reader :left
                attr_reader :right

                def initialize(name, string: name, left: nil, right: nil)
                    super(name)
                    raise "opens_indent_block unsupported on the left side" if left && left.opens_indent_block?
                    @string = string
                    @left = left
                    @right = right
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
                def expression
                    self if expression?
                end
                def infix?
                    left && right
                end
                def infix
                    self if infix?
                end
                def prefix?
                    !left && right
                end
                def prefix
                    self if prefix?
                end
                def postfix?
                    left && !right                    
                end
                def postfix
                    self if postfix?
                end

                def left_is_operand?
                    !left
                end
                def right_is_operand?
                    !right
                end

                def variants
                    [self]
                end
            end
        end
    end
end
