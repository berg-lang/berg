require_relative "expression"
require_relative "token"

module BergLang
    module Ast
        class StringLiteral < Expression
            include Token
        end
    end
end
