require_relative "operator"

module BergLang
    class Parser
        class IndentOperator < Operator
            attr_reader :indent

            def initialize(source_range, indent, operator_definitions)
                super(source_range, operator_definitions)
                @indent = indent
            end
        end
    end
end
