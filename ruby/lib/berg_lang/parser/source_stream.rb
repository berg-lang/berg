require_relative "../source_range"
require_relative "source_match"

module BergLang
    class Parser
        class SourceStream
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

            def ==(other)
                other.is_a?(SourceStream) && name == other.name && string == other.string
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
                SourceRange.new(self, at_index, at_index)
            end

            def to_location(index)
                line = nil
                column = nil
                (line_starts.size-1).downto(0) do |line_number|
                    line_start = line_starts[line_number]
                    if index >= line_start
                        line = line_number + 1
                        column = index - line_start + 1
                        break
                    end
                end
                [ line, column ]
            end

            def to_index(location)
                line, column = location
                line_starts[line-1] + (column-1)
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
end