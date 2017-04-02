require_relative "../token"
require_relative "../expression"

module Berg
    module Expressions
        #
        # Represents hexadecimal literals such as 0x1f10 and 0xDEADBEEF
        #
        class NumericLiteral < Expression
            include Token
        end
    end
end
