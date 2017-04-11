module BergLang
    module Token
        attr_reader :source_range

        def initialize(source_range)
            @source_range = source_range
        end

        def to_s
            source_range.string
        end
    end
end