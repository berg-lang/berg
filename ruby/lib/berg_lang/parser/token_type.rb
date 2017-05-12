module BergLang
    class Parser
        class TokenType
            attr_reader :name
            attr_accessor :whitespace
            attr_accessor :newline
            attr_accessor :expression
            attr_accessor :infix
            attr_accessor :prefix
            attr_accessor :postfix

            def initialize(name, whitespace: nil, newline: nil, expression: nil, infix: nil, prefix: nil, postfix: nil)
                @name = name
                @whitespace = whitespace
                @newline = newline
                @expression = expression
                @infix = infix
                @postfix = postfix
                @prefix = prefix
            end

            def infix?
                infix
            end
            def prefix?
                prefix
            end
            def expression?
                expression
            end
            def postfix?
                postfix
            end
            def whitespace?
                whitespace
            end
            def newline?
                newline
            end

            def variants
                [ expression, infix, prefix, postfix ].reject { |variant| variant.nil? }
            end
        end
    end
end
