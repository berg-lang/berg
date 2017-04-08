module Berg
    module Token
        attr_reader :match

        def initialize(match)
            @match = match
        end

        def input_range
            match.offset(0)
        end

        def to_s
            match.to_s
        end
    end
end