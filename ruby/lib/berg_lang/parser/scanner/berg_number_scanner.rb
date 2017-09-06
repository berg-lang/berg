module BergLang
    module Parser
        module BergNumberScanner
            # Included in BergScanner

            DECIMAL_DIGITS = "0".."9"
            HEXADECIMAL_DIGITS = [ "0".."9", "A".."F", "a".."f" ]
            OCTAL_DIGITS = "0".."7"
            BINARY_DIGITS = "0".."1"
            INTEGER_PREFIXES = {
                [ "x", "X" ] => [ HEXADECIMAL_DIGITS, hexadecimal_literal ],
                [ "b", "B" ] => [ BINARY_DIGITS, binary_literal ],
                [ "o", "O" ] => [ OCTAL_DIGITS, octal_literal ],
            }

            def scan_number
                # 0, 0.1234, 0xDEADBEEF, 0o777, 0b1011
                if consume_if "0"
                    scan_zero_prefix_number

                # 123, 123.456
                elsif consume_all DECIMAL_DIGITS
                    if consume_all IDENTIFIER_CHARACTERS
                        buffer << token(error_identifier_starts_with_number)
                    else
                        scan_trailing_float
                    end
                end
            end

            def scan_zero_prefix_number
                digits, literal_type = consume_if INTEGER_PREFIXES
                if digits
                    has_digits = consume_all digits
                    has_decimal = consume_all DECIMAL_DIGITS

                    # 0o777999xxx: identifier starts with number error
                    if consume_all IDENTIFIER_CHARACTERS
                        output_error error_identifier_starts_with_number

                    # 0o777999: decimal error
                    elsif has_decimal
                        buffer << error_token(error_decimal_digits_in_other_literal)

                    # 0xabc1243, 0o754, 0b01011
                    elsif has_digits
                        buffer << token(literal_type)

                        start_symbol!
                        has_dot = consume_if "."
                        if has_dot
                            # 0xabc1243.0194 (but NOT 0x123.ToString): non-decimal float error
                            if consume_all DECIMAL_DIGITS
                                if consume_if ["e","E"]
                                    consume_if ["+","-"]
                                end
                                # Just skip the rest, we'll be emitting a nice error anyway.
                                consume_if IDENTIFIER_CHARACTERS
                                buffer << error_token(error_non_decimal_integer_has_floating_point)

                            else
                                buffer << operators["."]
                            end
                        end

                    # 0x, 0o, 0b: missing digits error
                    else
                        buffer << error_token(error_missing_digits_after_prefix)
                    end

                elsif consume_all DECIMAL_DIGITS

                    # 0b2xxx: identifier starts with number
                    if consume_all IDENTIFIER_CHARACTERS
                        buffer << error_token(error_identifier_starts_with_number)

                    # 0239873492 error: looks too much like octal
                    else
                        buffer << error_token(error_integer_cannot_start_with_zero)
                    end

                # 0yyy: identifier starts with number
                elsif consume_all IDENTIFIER_CHARACTERS
                    buffer << error_token(error_identifier_starts_with_number)

                # 0 or 0.123
                else
                    buffer << token(integer_literal)
                    scan_trailing_float
                end
            end

            def scan_trailing_float
                if consume_if decimal_point_character

                    # 123.456
                    if consume_all decimal_digit_characters

                        # 123.456e
                        if consume_if exponent_prefix_character
                            output float_trailing_digits
                            output float_literal_exponent

                            # 123.456e+
                            if consume_if positive_exponent_character
                                output float_literal_exponent_positive

                            # 123.456e-
                            elsif consume_if negative_exponent_character
                                output float_literal_exponent_negative
                            end

                            if consume_if decimal_digit_characters
                                # 123.456e+789xxx
                                if consume_if identifier_characters
                                    output_error error_identifier_starts_with_number

                                # 123.456e+789
                                else
                                    output integer_literal
                                end

                            # 123.456e
                            # 123.456e+
                            # 123.456exxx
                            # 123.456e+xxx
                            else
                                consume_if identifier_characters
                                output_error error_missing_exponent
                            end

                        
                        # 123.456xxx
                        elsif consume_if identifier_characters
                            output_error error_identifier_starts_with_number, :postfix

                        # 123.456
                        else
                            output float_trailing_digits
                        end

                    # 123.ToString
                    else
                        output operators["."]
                    end
                end
            end

        end
    end
end
