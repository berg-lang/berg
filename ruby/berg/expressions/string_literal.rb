require_relative "../expression"
require_relative "../token"

module Berg
    module Expressions
        class StringLiteral < Expression
            include Token
        end
    end
end
