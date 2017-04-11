require_relative "../expression"
require_relative "../token"

module BergLang
    module Expressions
        #
        # Represents imaginary literals such as 205i, -10.05i and .5e-10i
        #
        class ImaginaryLiteral < Expression
            include Token
        end
    end
end
