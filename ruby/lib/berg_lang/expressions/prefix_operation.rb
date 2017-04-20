require_relative "../expression"

module BergLang
    module Expressions
        class PrefixOperation < Expression
            attr_reader :operator
            attr_reader :right

            def initialize(operator, right)
                @operator = operator
                @right = right
            end

            def source_range
                SourceRange.span(operator, right)
            end

            def to_s
                "(#{operator}#{right})"
            end
        end
    end
end
