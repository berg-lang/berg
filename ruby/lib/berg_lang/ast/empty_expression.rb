require_relative "expression"
require_relative "token"

module BergLang
    module Ast
        class EmptyExpression < Expression
            include Token
        end
    end
end