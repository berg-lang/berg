require_relative "../token"

module BergLang
    class Parser
        class Whitespace
            include Token

            def newline
                source_range.named_captures["newline"]
            end

            def indent
                source_range.named_captures["indent"]
            end
        end
    end
end
