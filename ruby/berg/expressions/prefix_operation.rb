require_relative "../expression"

module Berg
    module Expressions
        class PrefixOperation < Expression
            attr_reader :operator
            attr_reader :b

            def initialize(operator, b)
                @operator = operator
                @b = b
            end

            def input_range
                [ operator.input_range[0], b.input_range[1] ]
            end

            def to_s
                "#{operator}#{b}"
            end
        end
    end
end
