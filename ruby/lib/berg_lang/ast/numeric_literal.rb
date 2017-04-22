require_relative "expression"
require_relative "token"

module BergLang
    module Ast
        #
        # Represents all numeric literals (superclass)
        #
        class NumericLiteral < Expression
            include Token
        end
    end
end
