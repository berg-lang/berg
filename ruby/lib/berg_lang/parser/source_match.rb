require_relative "../source_range"

module BergLang
    class Parser
        class SourceMatch < SourceRange
            attr_reader :named_captures

            def initialize(source, offset, match)
                super(source, offset, offset+match.end(0))
                capture_ranges = (1...match.size).map do |index|
                    start_offset, end_offset = match.offset(index)
                    if start_offset
                        SourceRange.new(source, offset+start_offset, offset+end_offset)
                    end
                end
                @named_captures = match.names.zip(capture_ranges).to_h
            end

            def [](capture_name)
                named_captures[capture_name.to_s].string if named_captures[capture_name.to_s]
            end
        end
    end
end