require_relative "token"

module BergLang
    module Ast
        class UnrecognizedCharacter
            include Token
        end
    end
end