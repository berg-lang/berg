require_relative "numeric_literal"

module BergLang
    module Ast
        #
        # Represents floating point literals such as -10.05, .5e-10, and 1.5i
        #
        class FloatLiteral < NumericLiteral
        end
    end
end
