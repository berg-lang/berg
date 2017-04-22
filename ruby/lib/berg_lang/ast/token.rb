module BergLang
    module Ast
        module Token
            attr_reader :source_range

            def initialize(source_range)
                @source_range = source_range
            end

            def skip?
                false
            end

            def has_left_side?
                false
            end

            def has_right_side?
                false
            end

            def to_s
                source_range.string
            end
        end
    end
end