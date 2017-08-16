module BergLang
    module Parser
        class StringParser
            include Parser
            extend Grammar

            def initialize(parse_state)
                @parse_state = parse_state
            end

            def parse(parent_parser)
                loop do
                    # Read raw characters
                    stream.consume_until("\\", "$", "\r", "\n", "\"")

                    # Check for interpolation
                    if stream.peek == "$"
                        case stream.peek(2)[1]

                        # Interpolated expression: ${1 * 2}
                        when "{"
                            append_symbol_unless_empty(bare_string)
                            stream.consume(2)
                            append_symbol(interpolated_expression_start)
                            ExpressionParser.new(parse_state, self, allow_newline: false).parse

                        # Interpolated identifier: $MyFavoriteThing
                        when "A".."Z", "a".."z", "_"
                            append_symbol_unless_empty(bare_string)
                            stream.consume
                            append_symbol(interpolated_identifier)
                            stream.consume_all("A".."Z", "a".."z", "0".."9", "_")
                            append_symbol(identifier)

                        # Just a $ sign, perhaps followed by space or a number. Plain ol' raw text.
                        else
                            consume
                        end

                    # Backslash escapes
                    elsif read_backslash_escape

                    # End quote
                    elsif stream.consume_if("\"")
                        append_symbol_unless_empty(bare_string)
                        append_symbol(string_terminator)
                        return parent_parser

                    # Unterminated string! (\r, \n or EOF)
                    else
                        append_symbol_unless_empty(bare_string)
                        append_symbol(error_unterminated_string)
                        return parent_parser
                    end
                end
            end

            def read_backslash_escape
                if !stream.consume_if("\\")
                    return false
                end

                append_symbol_unless_empty(bare_string)

                if symbol_type = stream.consume_if(
                    "n" => newline_escape,
                    "r" => carriage_return_escape)
                    append_symbol(symbol_type)

                    return symbol_type
                end

                if stream.consume_if("u")
                    if stream.consume_if("{")
                        # Read n hex digits, then } or space, then hex digits ....
                    else
                        append_symbol(unicode_4_digit_escape)
                        # Read up to 4 hex digits
                    end
                elsif stream.consume_if("U")
                    append_symbol(unicode_8_digit_escape)
                    # Read up to 8 hex digits
                else
                    append_symbol(error_invalid_escape_sequence)
                end
            end

            def read_bare_string
                if stream.consume_until("\\", "\r", "\n")
                    append_symbol(bare_string)
                end
            end

            def read_backslash
                if stream.consume_if("\\")
                    append_symbol(bare_string)
                end
            end

            expression_symbol :string_end
            prefix_symbol :bare_string
            prefix_symbol :newline_escape
            prefix_symbol :backslash_escape
            prefix_symbol :double_quote_escape
            prefix_symbol :unicode_4_digit_escape
            prefix_symbol :unicode_8_digit_escape
            open_delimiter "\\u{", closed_by: 
            prefix_symbol "\\u"
            prefix_symbol "\\U"
            expression_symbol :unicode_hexadecimal_codepoint
            binary_symbol :unicode_sequence
            binary_symbol :unicode_escape_end
            define_symbol :hexadecimal_digits, prefix: true, expression: true
            prefix_symbol :hexadecimal_digits
        end
    end
end
