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
                @index += 1
                process_current_character
            end
            peek
        end

        def match(regex)
            match = regex.match(string[index..-1])
            if match
                @index += match.end(0)
                match
            end
        end

        def location(index)
            line = line_starts.size
            line_starts.reverse_each do |line_start|
                break if line_start > index
                line -= 1
            end
            column = index - line_starts[line-1] + 1
            [ line, column ]
        end

        private

        def process_current_character
            if string[index] == "\n"
                line_starts << index + 1
            end
        end
    end
end
