require_relative "scanner/buffered_scanner"
require "forwardable"

module BergLang
    module Parser
        class BergScanner
            include Scanner::BufferedScanner

            attr_reader :context
            attr_reader :grammar

            def initialize(context, stream, grammar)
                super(context, stream)
                @grammar = grammar
                @state = nil
            end

            def start
                stream.start
                syntax_tree.source_data.line_start_codepoint_indices << 0
            end

            def stop
                stream.stop
            end

            def scan
                # Emit indent if applicable
                if stream.index == 0
                    consume_all space_characters
                    output indent unless peek_if newline_strings
                end

                @symbol_start = stream.index

                case state
                when nil
                    scan_expression
                when :string
                    scan_string
                when :interpolated_expression
                    scan_interpolated_expression
                when :interpolated_expression_string
                    scan_interpolated_expression_string
                when :interpolated_expression_unicode_block
                    scan_interpolated_expression_unicode_block
                else
                    raise ApplicationError, "Unknown state #{state.inspect}!"
                end
            end

            private

            extend Forwardable

            attr_reader :state

            (BergGrammar.instance_methods - Object.instance_methods).each do |method|
                def_delegators :grammar, method
            end

            def_delegators :stream, :consume_if, :consume_unless, :consume_all, :consume_until
            def_delegators :stream, :peek_if, :peek_unless

            def symbol_range(start = nil)
                if start.nil?
                    return if symbol_start == stream.index
                    start = symbol_start
                end
                start...stream.index
            end

            # Append the given symbol to the buffer
            def output(symbol_type, start: nil)
                buffer << Symbol.new(symbol_type, symbol_range(start))
            end

            def output_error(error_type, operation_type: :expression, start: nil)
                error = Error.new(error_type, symbol_range(start))
                context.syntax_tree.errors << error
                output error.send(operation_type), start: start
            end

            def scan_expression
                # Skip space and emit indent
                consume_all space_characters
                while consume_newline
                    indent_start = stream.index
                    consume_all space_characters
                end
                output(indent, start: indent_start) if indent_start
                skip

                @symbol_start = stream.index

                # Comment
                if consume_if single_line_comment_start
                    consume_all single_line_comment_characters
                    context.syntax_tree.comment_data << [ range, nil ]
                    output single_line_comment

                # String
                elsif consume_if string_start
                    @state = :string
                    output string_start

                # 0, 0.1234, 0xDEADBEEF, 0o777, 0b1011
                elsif consume_if zero
                    scan_zero_prefix_number

                # 123, 123.456
                elsif consume_all decimal_digit_characters
                    if consume_all identifier_characters
                        output_error error_identifier_starts_with_number
                    else
                        scan_trailing_float
                    end

                # + - ! ( ) ......
                elsif operator = consume_if operators
                    output operator

                # Error: unrecognized character
                else
                    # TODO probably need to consume more than just the one character
                    # to give a better message, but what? Maybe check what Unicode
                    # category the character is in. Also don't have a good way to
                    # exclude all valid characters in the grammar, so this is still
                    # easier ...
                    consume
                    output_error error_unrecognized_character
                end
            end

            def consume_newline
                # Record the line start data
                if result = consume_if(newline_strings)
                    syntax_tree.source_data.line_start_codepoint_indices << stream.index
                end
                result
            end

            def scan_string
                # escapes
                if consume_if string_escape_start_character
                    # \" \\ \n \t
                    if escape = consume_if string_escape_characters
                        output escape

                    # \( Expression )
                    elsif consume_if string_interpolated_expression_start_character
                        output string_interpolated_expression_start
                        @state = :interpolated_expression

                    # \u, \U
                    elsif escape = consume_if string_unicode_escape_characters
                        scan_unicode_escape

                    # \? (unsupported escape character)
                    else
                        output_error error_unsupported_escape_character
                    end

                # " (string terminator)
                elsif consume_if string_end_character
                    output string_end
                    @state = :expression

                # line terminator: unterminated string
                elsif consume_if newline_characters
                    output string_end
                    output_error error_unterminated_string

                # normal string characters
                else
                    consume_until string_special_characters
                    output bare_string
                end
            end

            # \u{10 13 ABC 10FFFF feff}
            def scan_unicode_block
                # Skip spaces
                consume_all space_characters

                # FEFF, 10, 10FFFF
                if consume_all hexadecimal_digit_characters
                    output string_unicode_escape_codepoint

                # }
                elsif consume_if string_unicode_block_end_character
                    output string_unicode_block_end

                elsif (peek_if newline_strings) || (peek_if string_end)
                    output error_unterminated_unicode_block
                    @state = :expression

                else
                    output error_unrecognized_character_in_unicode_block
                    @state = :string
                end
            end

            def scan_interpolated_expression
                # Uh. TODO. Not sure how to end the interpolated expression yet,
                # since symbols aren't affected by tokenization and we need to know
                # what's closed what to know when to switch back to string mode :(
                scan_expression
                case state
                when :string
                    @state = :interpolated_expression_string
                when :unicode_block
                    @state = :interpolated_expression_unicode_block
                end
            end

            def scan_interpolated_expression_string
                if peek_if(newline_strings)
                    output error_unterminated_string
                    @state = :expression
                else
                    scan_string
                    case state
                    when :expression
                        @state = :interpolated_expression
                    when :unicode_block
                        @state = :interpolated_expression_unicode_block
                    end
                end
            end

            def scan_interpolated_expression_unicode_block
                if peek_if(newline_strings)
                    output error_unterminated_unicode_block
                    @state = :expression
                else
                    scan_unicode_block
                    case state
                    when :expression
                        @state = :interpolated_expression
                    when :string
                        @state = :interpolated_expression_string
                    end
                end
            end

            def scan_zero_prefix_number
                if literal_type = consume_if integer_prefixes
                    has_digits = consume_all integer_digits[literal_type]
                    has_decimal = consume_all decimal_digit_characters

                    # 0o777999xxx: identifier starts with number error
                    if consume_all identifier_characters
                        output_error error_identifier_starts_with_number

                    # 0o777999: decimal error
                    elsif has_decimal
                        output_error error_decimal_digits_in_other_literal

                    # 0xabc1243, 0o754, 0b01011
                    elsif has_digits
                        output literal_type

                        has_dot = consume_if decimal_point_character
                        if has_dot
                            # 0xabc1243.0194 (but NOT 0x123.ToString): non-decimal float error
                            if consume_all decimal_digit_characters
                                if consume_if exponent_prefix_character
                                    consume_if exponent_sign_characters
                                end
                                # Just skip the rest, we'll be emitting a nice error anyway.
                                consume_if identifier_characters
                                output_error error_non_decimal_integer_has_floating_point

                            else
                                output operators["."]
                            end
                        end

                    # 0x, 0o, 0b: missing digits error
                    else
                        output_error error_missing_digits_after_prefix
                    end

                elsif consume_all decimal_digit_characters

                    # 0b2xxx: identifier starts with number
                    if consume_all identifier_characters
                        output_error error_identifier_starts_with_number

                    # 0239873492 error: looks too much like octal
                    else
                        output_error error_integer_cannot_start_with_zero
                    end

                # 0yyy: identifier starts with number
                elsif consume_all identifier_characters
                    output_error error_identifier_starts_with_number

                # 0 or 0.123
                else
                    output integer_literal
                    scan_trailing_float
                end
            end

            def scan_unicode_escape
                # \u{10 13 FEFF 10FFFF ...} \U{10 13 FEFF 10FFFF ...}
                if consume_if string_unicode_block_start_character
                    output string_unicode_block_start_character
                    @state = :unicode_block

                # \u10 \uFEFF \U10 \UFEFF \U10FFFF...
                elsif consume_if hexadecimal_digit_characters
                    consume_if hexadecimal_digit_characters
                    consume_if hexadecimal_digit_characters
                    consume_if hexadecimal_digit_characters
                    if escape == string_unicode_8_escape_character
                        consume_if hexadecimal_digit_characters
                        consume_if hexadecimal_digit_characters
                        consume_if hexadecimal_digit_characters
                        consume_if hexadecimal_digit_characters
                    end
                    output escape

                else
                    output_error error_missing_unicode_escape_digits
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

            private

            def output(symbol, start)
                syntax_tree_builder.output symbol
                true
            end

            def output_error(symbol, start)
                syntax_tree_builder.output 
            end

            def output!(symbol)
                syntax_tree_builder.output!(symbol)
                true
            end
        end
    end
end