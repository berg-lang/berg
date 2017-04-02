require_relative "../token"

module Berg
    class Parser
        class Whitespace
            include Token

            def has_newline?
                match[:newline]
            end

            def indent
                match[:indent]
            end
        end
    end
end
