require_relative "../expression"

module BergLang
    module Expressions
        class InfixOperation < Expression
            attr_reader :left
            attr_reader :operator
            attr_reader :right

            def initialize(left, operator, right)
                @left = left
                @operator = operator
                @right = right
            end

            def source_range
                SourceRange.span(left, right)
            end

            def to_s
                "(#{left} #{operator} #{right})"
            end
        end
    end
end
