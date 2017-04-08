require_relative "../expression"

module Berg
    module Expressions
        class InfixOperation < Expression
            attr_reader :a
            attr_reader :operator
            attr_reader :b

            def initialize(a, operator, b)
                @a = a
                @operator = operator
                @b = b
            end

            def input_range
                [ a.input_range[0], b.input_range[1] ]
            end

            def to_s
                "(#{a} #{operator} #{b})"
            end
        end
    end
end
