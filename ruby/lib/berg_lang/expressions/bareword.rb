require_relative "../token"
require_relative "../expression"

module BergLang
    module Expressions
        class Bareword < Expression
            include Token
        end
    end
end