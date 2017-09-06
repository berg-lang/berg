module BergLang
    module Parser
        class Scanner
            class BergStringScanner
                # Included in BergScanner

                STRING_ESCAPE_CHARACTERS = {
                    '"' => string_escape_quote,
                    "\\" => string_escape_backslash,
                    "r" => string_escape_carriage_return,
                    "n" => string_escape_newline,
                    "t" => string_escape_tab,
                }

                def scan_string
                    # escapes
                    if consume_if "\\"
                        # \" \\ \n \t
                        if escape = consume_if STRING_ESCAPE_CHARACTERS
                            buffer << token(escape)

                        # \( Expression )
                        elsif consume_if "("
                            buffer << token(string_interpolated_expression_start)

                        # \u, \U
                        elsif escape = consume_if [ "u", "U" ]
                            scan_unicode_escape(escape)

                        # \? (unsupported escape character)
                        else
                            buffer << error_token(error_unsupported_escape_character)
                        end

                    # " (string terminator)
                    elsif consume_if '"'
                        buffer << token(string_end)

                    # normal string characters
                    else
                        consume_until [ '"', "\\", NEWLINE_CHARACTERS ]
                        buffer << token(bare_string)
                    end
                end

                # \u{10 13 ABC 10FFFF feff}
                def scan_unicode_block
                    # Skip spaces
                    consume_all SPACE_CHARACTERS

                    # FEFF, 10, 10FFFF
                    start_symbol!
                    if consume_all HEXADECIMAL_DIGITS
                        buffer << token(string_unicode_escape_codepoint)

                    # }
                    elsif consume_if "}"
                        output string_unicode_block_end

                    elsif (peek_if NEWLINE_CHARACTERS) || (peek_if '"')
                        output error_unterminated_unicode_block

                    else
                        output error_unrecognized_character_in_unicode_block
                    end
                end

                def scan_unicode_escape(escape)
                    # \u{10 13 FEFF 10FFFF ...} \U{10 13 FEFF 10FFFF ...}
                    if consume_if "{"
                        buffer << token(string_unicode_block_start)
                        consume_all SPACE_CHARACTERS
                        start_symbol!
                        while consume_if HEXADECIMAL_DIGITS
                            buffer << token_with_string(string_unicode_escape_codepoint)
                            consume_all SPACE_CHARACTERS
                            start_symbol!
                        end
                        if !consume_if "}"
                            if (peek_if NEWLINE_CHARACTERS) || (peek_if '"')
                                buffer << error_token(error_unterminated_unicode_block)
                            else
                                consume_until NEWLINE_CHARACTERS + [ '"', "}" ]
                                buffer << error_token(error_unrecognized_character_in_unicode_block)
                            end
                        end

                    # \u10 \uFEFF \U10 \UFEFF \U10FFFF...
                    elsif consume_if HEXADECIMAL_DIGITS
                        consume_if HEXADECIMAL_DIGITS
                        consume_if HEXADECIMAL_DIGITS
                        consume_if HEXADECIMAL_DIGITS
                        if escape == "U"
                            consume_if HEXADECIMAL_DIGITS
                            consume_if HEXADECIMAL_DIGITS
                            consume_if HEXADECIMAL_DIGITS
                            consume_if HEXADECIMAL_DIGITS
                        end
                        token_type = escape == "u" ? string_unicode_4_character_escape : string_unicode_8_character_escape
                        buffer << token(token_type)

                    else
                        buffer << token(error_missing_unicode_escape_digits
                    end
                end
            end
        end
    end
end
