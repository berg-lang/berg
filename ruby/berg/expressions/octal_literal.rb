require_relative "../expression"
require_relative "../token"

module Berg
    module Expressions
        #
        # Represents octal literals such as 01
        #
        class OctalLiteral < Expression
            include Token
        end
    end
end
