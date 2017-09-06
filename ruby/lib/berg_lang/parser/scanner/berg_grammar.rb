require_relative "grammar_definition"
require_relative "berg_scanner"

module BergLang
    module Parser
        class BergGrammar
            extend GrammarDefinition

            def create_scanner(context)
                BergScanner.new(context, self)
            end

            #
            # Whitespace / comments
            #
            symbol :space
            symbol :newline
            symbol :comments
            symbol :indent

            #
            # Literals
            #
            # Identifier
            expression :identifier
            # Integer literals
            expression :decimal_literal
            expression :hexadecimal_literal
            expression :octal_literal
            expression :infix_literal
            # Float literal parts
            postfix :float_trailing_digits, precedence: tight
            infix :float_literal_exponent, precedence: tight
            prefix :float_literal_exponent_positive, precedence: tight
            prefix :float_literal_exponent_negative, precedence: tight
            postfix :imaginary, precedence: tight+1

            # String literal parts
            prefix :string_raw, precedence: tight+1
            prefix :string_newline_escape, precedence: tight+1
            prefix :string_unicode_4_escape, precedence: tight+1
            prefix :string_unicode_8_escape, precedence: tight+1
            prefix :string_unicode_block_start, precedence: tight+1, next_operator: :string_unicode_block_end
            prefix :string_unicode_block_codepoint, precedence: tight
            infix :string_unicode_block_end, precedence: tight
            prefix :string_tab_escape, precedence: tight+1
            prefix :string_backslash_escape, precedence: tight+1
            prefix :string_quote_escape, precedence: tight+1
            prefix :string_interpolated_expression_start, next_operator: :string_interpolated_expression_end
            infix :string_interpolated_expression_start

            #
            # Inserted between any two open statement separators
            #
            expression :missing_operand

            #
            # Operators (each group is a new precedence level)
            #
            infix "."
            prefix "-", "+", "!"
            prefix "++", "--"
            postfix "--", "++"
            infix "*", "/", "%"=
            infix "+", "-"
            infix ">", ">=", "<", "<="
            infix "==", "!="
            postfix "+", "*", "?"
            infix "&&"
            infix "||", "??"
            infix :apply
            infix ":", direction: :right
            infix "=", "+=", "-=", "*=", "/=", "%=", "||=", "&&=", "??=", direction: :right
            infix ","
            # TODO unsure if this is the right spot for intersect/union. Maybe closer to - and +
            # infix "&"
            # infix "|"
            infix ";"

            # Precedence of these doesn't matter, they are just "loosest"
            open_delimiter "(", next_operator: ")", block_operator: true
            open_delimiter "{", next_operator: "}", block_operator: true
            open_delimiter "[", next_operator: "]", block_operator: true
            open_delimiter :string_start, next_operator: :string_end

            infix :apply_block, block_operator: true
            infix :extend, block_operator: true
            prefix :block, block_operator: true

            # Errors
            errors(
                error_identifier_starts_with_number: "names cannot begin with numbers. Perhaps you meant to place an operator between them?",
                error_integer_cannot_start_with_zero: "Integers cannot be prefaced with zero (to avoid confusion with octal numbers).",
                error_missing_digits_after_prefix: "Hexadecimal, octal and binary numbers must have at least one digit.",
                error_decimal_digits_in_other_literal: "Octal and binary numbers cannot contain decimal numbers.",
                error_missing_exponent: "Missing exponent after exponent prefix.",
                error_unrecognized_character: "Unrecognized character.",
                error_unterminated_string: "Unterminated string.",
                error_unsupported_escape_character: "Unsupported escape character.",
                error_unterminated_unicode_escape: "\u{ without ending }",
                error_missing_unicode_escape_digits: "\u or \U missing an escape character.",
                error_unrecognized_character_in_unicode_escape: "Unrecognized character in Unicode escape.",
            )

            def integer_prefixes
                @integer_prefixes ||= {
                    hexadecimal_prefix_character => hexadecimal_literal,
                    octal_prefix_character => octal_literal,
                    binary_prefix_character => binary_literal,
                }
            end

            def integer_digits
                @integer_digits ||= {
                    hexadecimal_literal => hexadecimal_digit_characters,
                    octal_literal => octal_digit_characters,
                    binary_literal => binary_digit_characters,
                }
            end

            def string_simple_escape_characters
                {
                    "n" => newline_escape,
                    "t" => tab_escape,
                    "\\" => backslash_escape,
                    "\"" => quote_escape,
                }
            end

            #
            # Character definitions
            #
            characters(
                # Space
                newline_characters: [ "\r\n", "\r", "\n" ],
                space_characters: [ " ", "\t" ],
                invisible_characters: [ :space_characters, :newline_start_characters ],

                # Identifiers
                identifier_start_characters: [ "A".."Z", "a".."z", "_" ],
                identifier_middle_characters: [ :identifier_start_character, :decimal_digit_characters ],

                # Ints
                zero_character: "0",
                decimal_digit_characters: "0".."9",
                hexadecimal_digit_characters: [ "0".."9", "a".."f", "A".."F" ],
                octal_digit_characters: [ "0".."7" ],
                infix_digit_characters: [ "0".."1" ],
                hexadecimal_prefix_character: [ "x", "X" ],
                octal_prefix_character: [ "o", "O" ],
                infix_prefix_character: [ "b", "B" ],

                # Floats
                decimal_point_character: ".",
                exponent_character: [ "e", "E" ],
                imaginary_postfix_character: [ "i", "I" ],
                positive_exponent_character: "+",
                negative_exponent_character: "-",
                exponent_sign_characters: [ :positive_exponent_character, :negative_exponent_character ],

                # Strings
                string_escape_start_character: "\\",
                string_interpolated_expression_start_character: "(",
                string_interpolated_expression_end_character: ")",
                string_unicode_4_escape_character: "u",
                string_unicode_8_escape_character: "U",
                string_unicode_block_start_character: "{",
                string_unicode_block_end_character: "}",
                string_unicode_block_separator_characters: :space_characters,
                string_start_character: "\"",
                string_end_character: "\"",
            )

            def string_special_characters
                @string_special_characters ||= newline_strings + [ string_escape_start_character ] + [ string_end_character ]
            end
        end
    end
end
