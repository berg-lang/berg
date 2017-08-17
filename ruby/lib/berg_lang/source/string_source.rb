require_relative "../parser/string_source_stream"

module BergLang
    module Source
        class StringSource
            attr_reader :name
            attr_reader :string

            def initialize(name, string)
                @name = name
                @string = string
            end

            def open
                Parser::StringSourceStream.new(self)
            end

            def ==(other)
                other.is_a?(self) && name == other.name && string == other.string
            end
        end
    end
end