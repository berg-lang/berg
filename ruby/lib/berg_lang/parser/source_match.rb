require_relative "../source_range"

module BergLang
    class Parser
        class SourceMatch < SourceRange
            def initialize(source, offset, match)
                super(source, offset, offset+match.end(0))
                @match = match
            end

            def [](capture_name)
                match[capture_name.to_s] if match[capture_name.to_s]
            end
        end
    end
end