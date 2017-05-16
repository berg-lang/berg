module BergLang
    class Parser
        #
        # Represents a source range (with possible match data).
        #
        class SourceRange
            attr_reader :syntax_tree
            attr_reader :begin
            attr_reader :end

            def initialize(syntax_tree, begin_offset, end_offset)
                raise "nooo" unless syntax_tree.is_a?(SyntaxTree)
                @syntax_tree = syntax_tree
                @begin = begin_offset
                @end = end_offset
            end

            def source
                syntax_tree.source
            end

            def ==(other)
                other.is_a?(SourceRange) && source == other.source && self.begin == other.begin && self.end == other.end
            end

            def ===(other)
                other.respond_to?(:source_range) && self == other.source_range
            end

            def size
                self.end - self.begin
            end

            #
            # Create a region that includes both ranges
            #
            def self.span(begin_range, end_range)
                raise "COMPILER ERROR: different sources in span!" if begin_range.source_range.syntax_tree != end_range.source_range.syntax_tree
                SourceRange.new(begin_range.source_range.syntax_tree, begin_range.source_range.begin, end_range.source_range.end)
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
                syntax_tree.location_for(self.begin)
            end

            def begin_line
                begin_location[0]
            end

            def begin_column
                begin_location[1]
            end

            def end_location
                syntax_tree.location_for(self.end-1) if self.end > self.begin
            end

            def end_line
                location = end_location
                location ? location[0] : nil
            end

            def end_column
                location = end_location
                location ? location[0] : nil
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
end