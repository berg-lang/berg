require_relative "../term_type"

module BergLang
    class Parser
        class TermType
            class Ambiguous < TermType
                attr_reader :expression
                attr_reader :infix
                attr_reader :prefix
                attr_reader :postfix
                attr_reader :filler

                def initialize(expression: nil, infix: nil, prefix: nil, postfix: nil, filler: nil)
                    @expression = expression
                    @infix = infix
                    @prefix = prefix
                    @postfix = postfix
                    @filler = filler
                    validate!
                end

                def filler?
                    filler
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
                    (expression || infix || prefix || postfix || filler).name
                end

                def string
                    (expression || infix || prefix || postfix || filler).string
                end

                private

                def validate!
                    raise "expression variant of #{name} must have no operands" if expression && !expression.expression?
                    raise "infix variant of #{name} must have left and right operands" if infix && !infix.infix?
                    raise "prefix variant of #{name} must have only right operand" if prefix && !prefix.prefix?
                    raise "postfix variant of #{name} must have only left operand" if postfix && !postfix.postfix?
                    raise "filler variant of #{name} must actually be filler, not #{filler.fixity}" if filler && !filler.filler?
                    raise "filler variant of #{name} will never be chosen" if filler && expression && infix && prefix && postfix

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
