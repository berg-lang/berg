require_relative "../token"

module BergLang
    class Parser
        class UnrecognizedCharacter
            include Token
        end
    end
end