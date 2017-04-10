require_relative "source_range"
require_relative "source_match"

module Berg
    class Source
        attr_reader :name
        attr_reader :string
        attr_reader :line_starts
        attr_reader :index

        def initialize(name, string)
            @name = name
            @string = string
            @line_starts = [0]
            @index = 0
            process_current_character
        end

        def eof?
            peek.nil?
        end

        def peek
            string[index]
        end

        def next
            if !eof?
                advance(1)
            end
            peek
        end

        def advance(num_characters)
            1.upto(num_characters) do
                @index += 1
                process_current_character
            end
        end

        def match(regex)
            match = regex.match(string[index..-1])
            if match
                start_index = index
                advance(match.end(0))
                SourceMatch.new(self, start_index, match)
            end
        end

        def create_empty_range(at_index=index)
            SourceRange.new(self, index, index)
        end

        def location(index)
            line = line_starts.size
            line_starts.reverse_each do |line_start|
                break if line_start > index
                line -= 1
            end
            column = index - line_starts[line-1] + 1
            [ line+1, column ]
        end

        def substr(before, after)
            string[before...after]
        end

        private

        def process_current_character
            if string[index] == "\n"
                line_starts << index + 1
            end
        end
    end
end
