require_relative "grammar_definition"

module BergLang
    module Parser
        class BergGrammar
            extend GrammarDefinition

            #
            # Whitespace / comments
            #
            symbol :newline, :indent, :space
            ambiguous :single_line_comment, :prefix, :suffix, precedence: tight

            #
            # Literals
            #
            # Identifier
            expression :identifier
            # Integer literals
            expression :decimal_literal,
            expression :hexadecimal_literal
            expression :octal_literal
            expression :binary_literal
            # Float literal parts
            postfix :float_trailing_digits, precedence: tight
            binary :float_literal_exponent, precedence: tight
            prefix :float_literal_exponent_positive, precedence: tight
            prefix :float_literal_exponent_negative, precedence: tight
            suffix :imaginary, precedence: tight+1

            #
            # Inserted between any two open statement separators
            #
            expression :empty_block

            #
            # Operators (each group is a new precedence level)
            #
            binary "."
            prefix "-", "+", "!"
            prefix "++", "--"
            suffix "--", "++"
            binary "*", "/", "%"=
            binary "+", "-"
            binary ">", ">=", "<", "<="
            binary "==", "!="
            suffix "+", "*", "?"
            binary "&&"
            binary "||", "??"
            binary ":"
            binary "=", "+=", "-=", "*=", "/=", "%=", "||=", "&&=", "??=", direction: :right
            binary ","
            # TODO unsure if this is the right spot for intersect/union. Maybe closer to - and +
            # binary "&"
            # binary "|"
            binary :apply, statement_break: :child
            binary ";", statement_break: :sibling
            binary :extend, statement_break: :sibling

            # Precedence of these doesn't matter, they are just "loosest"
            open_delimiter "(", closed_by: ")", statement_break: :child
            open_delimiter "{", closed_by: "}", statement_break: :child
            prefix :string_start, string: "\""

            #
            # Errors
            #
            # Numeric errors
            expression :error_identifier_starts_with_number,
                       :error_integer_cannot_start_with_zero,
                       :error_missing_digits_after_exponent,
                       :error_missing_digits_after_hexadecimal_prefix,
                       :error_missing_digits_after_octal_prefix,
                       :error_missing_digits_after_binary_prefix
            # Malformed syntax errors
            expression :error_missing_operand

            # Unexpected characters are unknown, so we make them ambiguous to have maximum chance of continuing
            # TODO recognize operator symbols vs. word characters and split expression vs. infix+prefix+suffix
            ambiguous :error_unexpected_character, :expression, :binary, :prefix, :suffix, precedence: tight

            #
            # Character definitions
            #
            characters(
                # Space
                newline_strings: [ "\r\n", :newline_start_characters ],
                newline_start_characters: [ "\r", "\n" ]
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
                binary_digit_characters: [ "0".."1" ],
                hexadecimal_prefix_character: [ "x", "X" ],
                octal_prefix_character: [ "o", "O" ],
                binary_prefix_character: [ "b", "B" ],

                # Floats
                decimal_point_character: ".",
                exponent_character: [ "e", "E" ],
                imaginary_suffix_character: [ "i", "I" ],
                positive_exponent_character: "+",
                negative_exponent_character: "-",
            )

            operators
        end
    end
end
