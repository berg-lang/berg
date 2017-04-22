require_relative "expression"
require_relative "../source_range"

module BergLang
    module Ast
        class PostfixOperation < Expression
            attr_reader :left
            attr_reader :operator

            def initialize(left, operator)
                @left = left
                @operator = operator
            end

            def source_range
                SourceRange.span(left, operator)
            end

            def to_s
                "(#{left}#{operator})"
            end
        end
    end
end
