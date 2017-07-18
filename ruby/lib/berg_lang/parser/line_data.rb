module BergLang
    class Parser
        class LineData
            attr_reader :source

            def initialize(source)
                @source = source
                @line_locations = []
                @line_indents = []
                @line_is_blank = []
            end

            def append_line(start_index, indent_index, line_is_blank)
                @line_locations << start_index
                @line_indents << (indent_index - start_index)
                @line_is_blank << line_is_blank
            end

            def line_for(source_index)
                return nil if line_locations.empty?
                if line_locations.size > 1
                    line = line_locations.size * source_index / line_locations[-1]
                else
                    line = 0
                end
                line -= 1 while line_locations[line] > source_index
                line += 1 while line_locations[line+1] && line_locations[line+1] >= source_index
                line + 1
            end

            def location_for(source_index)
                line = line_for(source_index)
                column = source_index - line_locations[line-1] + 1
                [ line, column ]
            end

            def index_of_line(line)
                raise "invalid line number #{line}" if line < 1 || line > line_locations.size
                line_locations[line - 1]
            end

            def indent_at_line(line)
                raise "invalid line number #{line}" if line < 1 || line > line_indents.size
                line_indents[line - 1]
            end

            def line_is_blank?(line)
                raise "invalid line number #{line}" if line < 1 || line > line_is_blank.size
                line_is_blank[line - 1]
            end

            def current_indent
                raise "No lines means no indent" if line_indents.empty?
                line_indents[-1]
            end

            private

            attr_reader :line_locations
            attr_reader :line_indents
            attr_reader :line_is_blank
        end
    end
end
