require_relative "../expression"
require_relative "../token"

module BergLang
    module Expressions
        #
        # Represents floating point literals such as -10.05, .5e-10, and 1.5i
        #
        class FloatLiteral < Expression
            include Token
        end
    end
end
