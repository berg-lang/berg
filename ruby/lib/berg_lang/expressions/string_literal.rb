require_relative "../expression"
require_relative "../token"

module BergLang
    module Expressions
        class StringLiteral < Expression
            include Token
        end
    end
end
