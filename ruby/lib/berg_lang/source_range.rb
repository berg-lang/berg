module BergLang
    #
    # Represents a source range (with possible match data).
    #
    class SourceRange
        attr_reader :source
        attr_reader :begin
        attr_reader :end

        def initialize(source, begin_offset, end_offset)
            @source = source
            @begin = begin_offset
            @end = end_offset
        end

        #
        # Create a region that includes both ranges
        #
        def self.span(begin_range, end_range)
            raise "COMPILER ERROR: different sources in span!" if begin_range.source_range.source != end_range.source_range.source
            SourceRange.new(begin_range.source_range.source, begin_range.source_range.begin, end_range.source_range.end)
        end

        def string
            source.substr(self.begin, self.end)
        end

        def source_range
            self
        end

        def source_name
            source.name
        end

        def begin_location
            source.location(self.begin)
        end

        def end_location
            source.location(self.end-1)
        end

        def to_s
            result = "#{source_name}:#{range_string}"
        end

        def range_string
            result = "#{begin_location[0]}:#{begin_location[1]}"
            if end_location
                if end_location[0] != begin_location[0]
                    result << "-#{end_location[0]}:#{end_location[1]}"
                elsif end_location[1] != begin_location[1]
                    result << "-#{end_location[1]}"
                end
            end
            result
        end

        def lines_string
            result = "#{source_name}:#{begin_location[0]}"
            result << "-#{end_location[0]}" if end_location
            result
        end
    end
end