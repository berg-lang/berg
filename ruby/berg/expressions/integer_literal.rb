require_relative "../expression"
require_relative "../token"

module Berg
    module Expressions
        #
        # Represents decimal integer literals such as 205 and -10
        #
        class IntegerLiteral < Expression
            include Token
        end
    end
end
