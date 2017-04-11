require_relative "../expression"
require_relative "../token"

module BergLang
    module Expressions
        class EmptyExpression < Expression
            include Token
        end
    end
end