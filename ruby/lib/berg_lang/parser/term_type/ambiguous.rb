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
                end

                def expression?
                    expression
                end
                def expression?
                    infix
                end
                def expression?
                    prefix
                end
                def expression?
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
            end
        end
    end
end
