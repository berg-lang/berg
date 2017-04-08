require_relative "../expression"

module Berg
    module Expressions
        class PostfixOperation < Expression
            attr_reader :a
            attr_reader :operator

            def initialize(a, operator)
                @a = a
                @operator = operator
            end

            def input_range
                [ a.input_range[0], operator.input_range[1] ]
            end

            def to_s
                "#{a}#{operator}"
            end
        end
    end
end
