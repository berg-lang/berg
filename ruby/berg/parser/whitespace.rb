require_relative "../token"

module Berg
    class Parser
        class Whitespace
            include Token

            def has_newline?
                source_range[:newline]
            end

            def indent
                source_range[:indent]
            end
        end
    end
end
