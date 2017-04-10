require_relative "../expression"

module Berg
    module Expressions
        class DelimitedOperation < Expression
            attr_reader :start_delimiter
            attr_reader :expression
            attr_reader :end_delimiter

            def source_range
                SourceRange.span(start_delimiter, end_delimiter)
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
