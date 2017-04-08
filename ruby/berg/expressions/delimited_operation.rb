require_relative "../expression"

module Berg
    module Expressions
        class DelimitedOperation < Expression
            attr_reader :start_delimiter
            attr_reader :expression
            attr_reader :end_delimiter

            def input_range
                [ start_delimiter.input_range[0], end_delimiter.input_range[1] ]
            end

            def initialize(start_delimiter, expression, end_delimiter)
                @start_delimiter = start_delimiter
                @expression = expression
                @end_delimiter = end_delimiter
            end

            def to_s
                "#{start_delimiter}#{expression}#{end_delimiter}"
            end
        end
    end
end
