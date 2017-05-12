module BergLang
    class Parser
        #
        # Token parsed from 
        #
        class Token
            attr_accessor :leading_space
            attr_accessor :leading_newline
            attr_accessor :indent_start
            attr_accessor :indent_end
            attr_accessor :start
            attr_accessor :type
            attr_accessor :end
            attr_accessor :trailing_newline
            attr_accessor :trailing_space

            def initialize(leading_space, leading_newline, indent_start, indent_end, start, type, token_end, trailing_newline, trailing_space)
                @leading_space = leading_space
                @leading_newline = leading_newline
                @indent_start = indent_start
                @indent_end = indent_end
                @start = start
                @type = type
                @end = token_end
                @trailing_newline = trailing_newline
                @trailing_space = trailing_space
            end

            def indent_size
                indent_end - indent_start if indent_start
            end

            def leading_newline?
                leading_newline
            end
            def leading_space?
                leading_space < start
            end
            def trailing_newline?
                trailing_newline
            end
            def trailing_space?
                trailing_space > self.end
            end

            def left_needs_operator?
                if !(infix || postfix)
                    true
                elsif !(expression || prefix)
                    false
                else
                    nil
                end
            end
            def right_needs_operator?
                if !(infix || prefix)
                    true
                elsif !(expression || postfix)
                    false
                else
                    nil
                end
            end
            def left_is_operator?
                case left_needs_operator?
                when true
                    false
                when false
                    true
                end
            end
            def right_is_operator?
                case right_needs_operator?
                when true
                    false
                when false
                    true
                end
            end

            def infix
                defined?(@infix) ? @infix : type.infix
            end
            def infix=(value)
                @infix = value
            end
            def postfix
                type.postfix
            end
            def prefix
                type.prefix
            end
            def infix
                type.expression
            end

            def infix?
                infix
            end
            def postfix?
                postfix
            end
            def prefix?
                prefix
            end
            def expression?
                expression
            end
        end
    end
end