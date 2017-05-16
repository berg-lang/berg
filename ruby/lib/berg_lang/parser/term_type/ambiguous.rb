require_relative "../term_type"

module BergLang
    class Parser
        class TermType
            class Ambiguous < TermType
                attr_reader :expression
                attr_reader :infix
                attr_reader :prefix
                attr_reader :postfix

                def initialize(expression: nil, infix: nil, prefix: nil, postfix: nil)
                    @expression = expression
                    @infix = infix
                    @prefix = prefix
                    @postfix = postfix
                    validate!
                end

                def expression?
                    expression
                end
                def infix?
                    infix
                end
                def prefix?
                    prefix
                end
                def postfix?
                    postfix
                end

                def name
                    (expression || infix || prefix || postfix).name
                end

                def string
                    (expression || infix || prefix || postfix).string
                end

                def variants
                    [ expression, infix, prefix, postfix ].reject { |variant| variant.nil? }
                end

                private

                def validate!
                    raise "expression must have no operands" if expression && !expression.expression?
                    raise "infix must have left and right operands" if infix && !infix.infix?
                    raise "prefix must have only right operand" if prefix && !prefix.prefix?
                    raise "postfix must have only left operand" if postfix && !postfix.postfix?
                    if (infix && infix.right.opens_indent_block?) || (prefix && prefix.right.opens_indent_block?)
                        raise "#{name} has an infix or prefix that opens an indent block, but also has a postfix or expression variant. This ambiguity is not supported by the parser." if postfix || expression
                        # If both infix and prefix are defined, and only one opens an indent block, we're still OK,
                        # because infix and prefix together are not ambiguous (we know which to pick without looking
                        # ahead to see what's on the right).
                    end
                end
            end
        end
    end
end
