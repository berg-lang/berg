require_relative "expression"
require_relative "../source_range"

module BergLang
    module Ast
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
