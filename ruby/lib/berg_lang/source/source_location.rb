require_relative "source_data"

module BergLang
    module Source
        #
        # Represents a source range (with possible match data).
        #
        class SourceLocation
            attr_reader :source_data
            attr_reader :codepoint_index

            def initialize(source_data, codepoint_index)
                raise "source_data must be SourceData" unless source_data.is_a?(SourceData)
                @source_data = source_data
                @codepoint_index = codepoint_index
            end

            def source
                source_data.source
            end

            def line
                source_data.line_for_codepoint_index(codepoint_index)
            end

            def column
                source_data.column_for_codepoint_index(codepoint_index)
            end

            include Comparable

            def ==(other)
                other.is_a?(SourceLocation) && source == other.source && self.codepoint_index == other.codepoint_index
            end

            def <=>(other)
                return nil unless other.is_a?(SourceLocation) && source == other.source
                self.codepoint_index <=> other.codepoint_index
            end
        end
    end
end
