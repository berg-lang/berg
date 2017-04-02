require_relative "../expression"
require_relative "../token"

module Berg
    module Expressions
        class EmptyExpression < Expression
            include Token
        end
    end
end