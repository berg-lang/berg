require_relative "../parser/scanner/bbuffered_stream"

module BergLang
    module Source
        class StringSourceStream
            include Parser::Scanner::BufferedStream

            def initialize(*args)
                super
                @string = source.string
            end

            def substr(range)
                string[range]
            end

            def advance(amount=1)
                remaining_amount = string.size - index
                amount = remaining_amount if remaining_amount < amount
                @index += amount
                amount
            end

            private

            attr_reader :string
        end
    end
end
