require_relative "source_data"
require_relative "source_location"

module BergLang
    module Source
        #
        # Represents a source range (with possible match data).
        #
        class SourceRange
            attr_reader :source_data
            attr_reader :codepoint_range

            def initialize(source_data, codepoint_range)
                raise "source_data must be SourceData" unless source_data.is_a?(SourceData)
                @source_data = source_data
                @codepoint_range = codepoint_range
            end

            def source
                source_data.source
            end

            def ==(other)
                other.is_a?(SourceRange) && source == other.source && self.codepoint_range == other.codepoint_range
            end

            def ===(other)
                other.respond_to?(:codepoint_range) && self == other.codepoint_range
            end

            def size
                codepoint_range.size
            end

            def start
                SourceLocation.new(source_data, codepoint_range.start)
            end
            def end
                SourceLocation.new(source_data, codepoint_range.end)
            end

            #
            # Create a region that includes both ranges (and anything in the middle)
            #
            def span(other)
                raise "COMPILER ERROR: different sources in span!" if source_data != other.source_data
                min = [ codepoint_range.start, other.codepoint_range.start ].min
                max = [ codepoint_range.end, other.codepoint_range.end ].max
                SourceRange.new(source_data, min, max)
            end

            def to_s
                result = "#{source_name}:#{range_string}"
            end

            def range_string
                result = "#{self.start.line}:#{self.start.column}"
                if self.start.line != self.end.line
                    result << "-#{self.end.line}:#{self.end.column}"
                elsif self.start.column != self.end.column
                    result << "-#{self.end.column}"
                end
                result
            end

            def lines_string
                result = "#{source_name}:#{self.start.line}"
                result << "-#{self.end.line}" if self.start.line != self.end.line
                result
            end
        end
    end
end