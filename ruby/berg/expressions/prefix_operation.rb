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

            def to_s
                "#{operator}#{b}"
            end
        end
    end
end
