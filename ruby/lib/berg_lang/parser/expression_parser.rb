require_relative "../parser"
require_relative "grammar"

module BergLang
    module Parser
        class ExpressionParser
            include Parser

            attr_reader :parse_state

            def initialize(parse_state, parent_parser)
                @parse_state = parse_state
                @parent_parser = parent_parser
            end

            def parse
                while true
                    case peek

                    # Space
                    when " ", "\t"
                        consume_space
                        append space

                    # Newline (and indent)
                    when "\r", "\n"
                        read_newline

                    # Single-line comment
                    when "#"
                        consume_until_newline
                        append single_line_comment

                    # String
                    when string_start.string
                        append string_start
                        return string_parser

                    when "0"
                        read_zero_prefix_number

                    when "1".."9"
                        consume_digits
                        read_trailing_float

                        read_decimal_number

                    else
                        # Operator
                        operator = consume_if(operators)
                        if operator
                            append symbol

                        # Unexpected character
                        else
                            # TODO probably need to consume more than just the one character to give a better message, but
                            # what? Maybe check whatUnicode category the character is in.
                            consume
                            append error_unexpected_character
                        end
                    end
                end
                read_operator ||
                read_identifier ||
                read_string ||
                read_decimal_number ||
                read_hexadecimal_number ||
                read_octal_number ||
                read_binary_number ||
                error_unexpected_character
            end

            def visible?(codepoint)
                ![" ", "\t", "\r", "\n"].include?(codepoint)
            end

            def consume_space
                consume_all(" ", "\t")
            end

            def consume_digits

            end

            def consume_binary_digits
                consume_all(binary_digit_characters)
            end

            def consume_octal_digits
                consume_all(octal_digit_characters)
            end

            def consume_hexadecimal_digits
                consume_all(*hexadecimal_digit_characters)
            end

            def consume_until_newline
                consume_until(newline_start_characters)
            end

            def read_newline
                # Have to consume \r\n as well
                if consume_if(*newline_strings)
                    append newline
                    # Read indent
                    consume_all(" ", "\t")
                    if visible?(peek)
                        append indent
                    else
                        append space
                    end
                end
            end

            def read_zero_prefix_number
                consume
                case peek
                when "x", "X"
                    if consume_hexadecimal_digits
                        append hexadecimal_prefix
                    else
                        append error_missing_digits_after_hexadecimal_prefix
                    end
                when "b", "B"
                    if consume_binary_digits
                        append octal_literal
                    else
                        append error_missing_digits_after_octal_prefix
                    end
                when "o", "O"
                    if consume_octal_digits
                        append octal_literal
                    else
                        append error_missing_digits_after_octal_prefix
                    end
                when decimal_digit_characters
                    consume_decimal_digits
                    if consume_literal
                    error_integer_cannot_start_with_zero
                else
                    append integer_literal
                    read_trailing_float
                end
            end

            def read_comment
                if consume_if("#")
                    consume_until([ "\r", "\n" ])
                    append stream
                end
            end

            def read_identifier
                if consume_if("A"..."Z", "a"..."z", "_")
                    consume_all("A"..."Z", "a"..."z", "0"..."9", "_")
                    append identifier
                end
            end

            # Hexadecimal: 0x1A
            def read_hexadecimal_number
                if consume_if("0x", "0X")
                    if consume_all("0".."9", "A".."F", "a".."f")
                        append hexadecimal_literal
                    else
                        append error_missing_number_after_hexadecimal_prefix
                    end
                end
            end

            # Octal: 0o777
            def read_octal_number
                if consume_if("0b", "0B")
                    if consume_all("0".."7")
                        append octal_literal
                    else
                        append error_missing_number_after_octal_prefix
                    end
                end
            end

            # Binary: 0b111
            def read_binary_number
                if consume_if("0b", "0B")
                    if consume_all("0".."1")
                        append binary_literal
                    else
                        append error_missing_number_after_binary_prefix
                    end
                end
            end

            def read_decimal_number
                if consume_if("00".."09")
                    # "old style" octal number: 0777
                    consume_all("0".."9")
                    return append error_number_cannot_start_with_zero
                end

                if consume_all("0".."9")
                    # Float or integer?
                    if consume_if(".0"..".9")
                        consume_all("0".."9")
                        append float_literal
                    else
                        append integer_literal
                    end
                else
                    return false
                end

                # Exponent: 1e-29
                if consume_if([ "e", "E" ])
                    append float_exponent_operator
                    if consume_if("+")
                        append positive_sign
                    elsif consume_if("-")
                        append negative_sign
                    end
                    if consume_all("0".."9")
                        append integer_literal
                    else
                        # Error: Missing digits after exponent: 1e
                        append error_missing_digits_after_exponent
                    end
                end

                # Imaginary: 420i
                if consume_if("i", "I")
                    append imaginary_suffix
                end

                # Error: identifier starts with number: 123abc, 1.23abc, 1e-23abc
                if consume_all("A".."Z", "a".."z", "0".."9", "_")
                    append error_identifier_starts_with_number
                end

                return true
            end

            def read_operator
                symbol = consume_if(operators)
                if symbol
                    append symbol
                end
            end

            private

            def append symbol
                syntax_tree_builder.append symbol
                true
            end

            def append!(symbol)
                syntax_tree_builder.append!(symbol)
                true
            end
        end
    end
end