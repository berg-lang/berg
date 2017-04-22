require_relative "numeric_literal"

module BergLang
    module Ast
        #
        # Represents hexadecimal literals such as 0x1f10 and 0xDEADBEEF
        #
        class HexadecimalLiteral < NumericLiteral
        end
    end
end
