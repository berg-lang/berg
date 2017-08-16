require_relative "source_stream"

module BergLang
    module Source
        class StringSourceStream
            include SourceStream

            attr_reader :source
            attr_reader :codepoint_index

            def initialize(source)
                @source = source
                @codepoint_index = 0
            end

            def peek(size=1)
                actual_lookahead = source.string.size - codepoint_index
                actual_lookahead = size if size < actual_lookahead
                return nil unless actual_lookahead >= 1
                source.string[codepoint_index...codepoint_index+actual_lookahead]
            end

            def consume(size=1, append_to: nil)
                actual_lookahead = source.string.size - codepoint_index
                actual_lookahead = size if size < actual_lookahead
                return nil unless actual_lookahead >= 1
                if append_to
                    append_to << source.string[codepoint_index...codepoint_index+actual_lookahead]
                end
                @codepoint_index += actual_lookahead
                actual_lookahead
            end
        end
    end
end
