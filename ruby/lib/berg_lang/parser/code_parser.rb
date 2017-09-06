module BergLang
    module Parser
        class CodeParser
            def initialize(context)
                @context = context
            end

            SPACE_CHARACTERS = ""
            NEWLINE_CHARACTERS = [ "\r\n", "\r", "\n" ]

            def parse_block(margin)
                expression = nil
                while indent = scanner.read_space
                    next if scanner.skip_newline

                    if prev_token = read_token(nil)
                    end

                    if prev_token = read_token(left_connecting_operator)
                        while token = read_token(prev_token)
                            
                        end
                        if last_token.right_is_operator?
                        else
                            append_child(last_token)
                        end
                    end

                    scanner.skip_newline
                end
            end

            private

            SPACE_CHARACTERS = [ " ", "\t" ]
            NEWLINE_CHARACTERS = [ "\r\n", "\r", "\n" ]
            DIGITS = "0".."9"
            IDENTIFIER_CHARACTERS = [ "A".."Z", "a".."z", "_" ]
            INVISIBLE_CHARACTERS = []
            NON_OPERATOR_CHARACTERS = SPACE_CHARACTERS + NEWLINE_CHARACTERS + DIGITS + IDENTIFIER_CHARACTERS + INVISIBLE_CHARACTERS

            def read_token
                token_start = stream.index
                token_type = read_raw_token
                token_type = read_number(token_start) if token_type == :number
                return nil if token_type.nil?
                [ token_start, token_type, token_string(token_start) ]
            end

            def peek_token_type
                case stream.peek
                when nil
                    nil
                when *NEWLINE_CHARACTERS
                    :newline
                when *DELIMITERS
                    :delimiter
                when *DIGITS
                    :number
                when *IDENTIFIER_CHARACTERS
                    :identifier
                when *INVISIBLE_CHARACTERS
                    :invisible
                else
                    :operator
                end
            end

            def read_raw_token
                token_type = peek_token_type
                case token_type
                when :newline, :delimiter
                    stream.read
                when :space, :invisible, :operator, :number
                    stream.read while peek_token_type == token_type
                when :identifier
                    consume_identifier
                else
                    raise "Unknown token type #{token_type}"
                end
                token_type
            end

            include ExpressionTokenizer

            def consume_identifier
                if [ :number, :identifier ].include?(peek_token_type)
                    stream.read while [ :number, :identifier ].include?(peek_token_type)
                    true
                end
            end

            def consume_number
                if peek_token_type == :number
                    stream.read while peek_token_type == :number
                    true
                end
            end
                
            def read_operator
            end

            attr_reader :token_cache
            def read_token(prev_token=nil)
                token = token_cache.dequeue
                token ||= begin
                    scanner.scan(token_cache, prev_token)
                    token_cache.dequeue
                end
            end

            #
            # Specific to Berg
            #


            def read_space
                start_symbol!
                return symbol_string if skip_space
            end
            def skip_space
                consume_all(SPACE_CHARACTERS)
            end
            def skip_newline
                if consume(NEWLINE_CHARACTERS)
                    source_data.line_starts << stream.index
                end
            end

            #
            # Generic "read/skip/buffer symbol" stuff
            #

            attr_reader :symbol_start
            def peek_before_symbol
                stream.substr(symbol_start-1)
            end
            def start_symbol!
                stream.release(symbol_start-1)
                @symbol_start = stream.index
            end
            def symbol_string
                stream.substr(symbol_start...stream.index)
            end

            def consume(filter)
                stream.consume_if(filter)
            end
            def consume_all(filter)
                stream.consume_all(filter)
            end
        end
    end
end