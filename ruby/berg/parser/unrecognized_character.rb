require_relative "../token"

module Berg
    class Parser
        class UnrecognizedCharacter
            include Token
        end
    end
end