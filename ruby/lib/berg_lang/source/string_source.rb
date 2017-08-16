require "unicode"
require_relative "string_source_stream"

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
            other.is_a?(self) && name == other.name && string == other.string
        end
    end
end