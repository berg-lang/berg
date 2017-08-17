module BergLang
    module Parser
        class Error
            attr_reader :error_type
            attr_reader :range
            attr_reader :suggested_changes

            def initialize(error_type, range, suggested_changes=[])
                @error_type = error_type
                @range = range
                @suggested_changes = suggested_changes
            end

            def string
                error_type.message.replace("\\(TokenString)", range.string)
            end

            def infix
                @infix ||= TokenType.new(error_type, :infix, precedence: 1)
            end
            def prefix
                @prefix ||= TokenType.new(error_type, :prefix, precedence: 1)
            end
            def postfix
                @postfix ||= TokenType.new(error_type, :postfix, precedence: 1)
            end
            def expression
                @expression ||= TokenType.new(error_type, :expression, precedence: 1)
            end
        end
    end
end