module BergLang
    module Parser
        class CodeParser
            module NumberTokenizer
                def read_number_literal
                    if token_string == "0"
                        case stream.peek
                        when "x", "X"
                            token_type = read_prefix_number(:hexadecimal_literal, "0".."9", "A".."F", "a".."f")
                        when "o", "O"
                            token_type = read_prefix_number(:octal_literal, "0".."7")
                        when "b", "B"
                            token_type = read_prefix_number(:binary_literal, "0".."1")
                        end
                    end

                    if !token_type
                        case stream.peek
                        when "."
                            if DIGITS.include?(stream.peek(2))
                                token_type = read_float_decimal
                            else
                                token_type = :integer_literal
                            end
                        when "e", "E"
                            token_type = read_float_exponent
                        when "i", "I"
                            read_imaginary_suffix
                        else
                            token_type = :integer_literal
                        end
                    end

                    token_type = error_identifier_starts_with_number if consume_identifier

                    token_type
                end

                def read_prefix_number(digits, token_type)
                    stream.read # Skip the x/o/b
                    stream.read while digits.include?(stream.peek)
                    token_type = error_decimal_digits_in_octal_or_binary if consume_number
                    token_type
                end

                def read_float_decimal
                    stream.read # Clear the "."
                    consume_number # We already know there's at least one digit

                    case stream.peek
                    when "e", "E"
                        read_float_exponent
                    when "i", "I"
                        read_imaginary_suffix
                    else
                        :float_literal
                    end
                end

                def read_float_exponent
                    stream.read # Clear the "e"
                    if [ "-", "+" ].include?(stream.peek)
                        stream.read
                    end
                    if !consume_number
                        return error_missing_exponent_in_float_literal
                    end

                    case stream.peek
                    when "i", "I"
                        read_imaginary_suffix
                    else
                        :float_literal
                    end
                end

                def read_imaginary_suffix
                    stream.read # Clear the "i"
                    :imaginary_literal
                end
            end
        end
    end
end