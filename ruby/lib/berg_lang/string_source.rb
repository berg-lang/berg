require_relative "parser/source_range"

module BergLang
    class StringSource
        attr_reader :name
        attr_reader :string

        def initialize(name, string)
            @name = name
            @string = string
        end

        def open
            StringSourceStream.new(self)
        end

        def ==(other)
            other.is_a?(StringSource) && name == other.name && string == other.string
        end

        def substr(before, after)
            string[before...after]
        end

        class StringSourceStream
            attr_reader :source
            attr_reader :index

            def initialize(source)
                @source = source
                @index = 0
            end

            def eof?
                peek.nil?
            end

            def peek
                source.string[index]
            end

            def next
                advance(1) unless eof?
                peek
            end

            def advance(num_characters)
                @index += num_characters
            end

            def match(regex)
                match = regex.match(source.string[index..-1])
                if match
                    advance(match.end(0))
                    match
                end
            end
        end
    end
end