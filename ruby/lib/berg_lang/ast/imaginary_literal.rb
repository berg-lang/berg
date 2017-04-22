require_relative "numeric_literal"

module BergLang
    module Ast
        #
        # Represents imaginary literals such as 205i, -10.05i and .5e-10i
        #
        class ImaginaryLiteral < NumericLiteral
        end
    end
end
