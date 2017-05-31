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
                    super(name)
                end

                def name
                    (expression || infix || prefix || postfix).name
                end

                def string
                    (expression || infix || prefix || postfix).string
                end
            end
        end
    end
end
