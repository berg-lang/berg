require_relative "syntax_errors"

module BergLang
    class Parser
        #
        # Scans for tokens from the stream (whitespace, numbers, strings, words).
        #
        class Scanner
            attr_reader :grammar
            attr_reader :stream
            attr_reader :output

            def initialize(grammar, stream, output)
                @grammar = grammar
                @stream = stream
                @output = output
            end

            def eof?
                stream.eof?
            end

            def index
                stream.index
            end

            def next
                scan_token
            end

            def next_is_space?
                !!(stream.peek =~ /\s/) || stream.peek == '#'
            end

            private

            def scan_token
                return nil if stream.eof?

                token = parse_space || parse_number || parse_operator || parse_string || parse_bareword
                if !token
                    raise unrecognized_character(create_token(stream.next, grammar.unrecognized_character))
                end
                token
            end

            include SyntaxErrors

            WHITESPACE = /\A[[:blank:]]+/

            def parse_space
                if stream.match(/\A\r?\n/)
                    grammar.newline
                elsif stream.match(WHITESPACE)
                    grammar.whitespace
                elsif stream.match(/\A#[^\n]*/)
                    grammar.comment
                end
            end

            def parse_operator
                if match = stream.match(operators_regexp)
                    grammar.tokens[match.to_s]
                end
            end

            def parse_bareword
                if stream.match(/\A(\w|[_$])+/)
                    grammar.bareword
                end
            end

            def parse_string
                if stream.peek == '"'
                    if stream.match(/\A"(\\.|[^\\"])*"/m)
                        grammar.string_literal
                    else
                        start_index = index
                        raise internal_error("Expected to skip unclosed string, could not parse") unless stream.match(/\A"(\\.|[^\\"])*/m)
                        raise unclosed_string(SourceRange.new(syntax_tree, start_index, index))
                    end
                end
            end

            def parse_number
                # Handle hex literals (0xDEADBEEF)
                # prefix integer
                start_index = index
                if stream.match(/\A(?<prefix>0[xX])(?<integer>(\d|[A-Fa-f])+)/)
                    illegal_word_characters = stream.match /\A(\w|[_$])+/
                    # Word characters immediately following a number is illegal.
                    if illegal_word_characters
                        raise variable_name_starting_with_an_integer(SourceRange.new(syntax_tree, start_index, index), illegal_word_characters)
                    end
                    return grammar.hexadecimal_literal
                end

                #
                # Handle floats, imaginaries and integers (hex is later in this function)
                #
                # integer (. decimal)? (e expsign? exponent)? i?
                start_index = index
                if match = stream.match(/\A(?<integer>\d+)((\.)(?<decimal>\d+))?((e)(?<expsign>[-+])?(?<exp>\d+))?(?<imaginary>i)?/i)
                    illegal_word_characters = stream.match /\A(\w|[_$])+/
                    # Word characters immediately following a number is illegal.
                    if illegal_word_characters
                        range = SourceRange.new(syntax_tree, start_index, index)
                        if !match[:exp] && !match[:imaginary] && illegal_word_characters.string.downcase == "e"
                            raise empty_exponent(range)
                        elsif match[:decimal]
                            raise float_with_trailing_identifier(range)
                        else
                            raise variable_name_starting_with_an_integer(range)
                        end
                    end

                    is_imaginary = match[:imaginary]
                    is_float = match[:decimal] || match[:exp]
                    is_octal = !is_float && match[:integer] && match[:integer].length > 1 && match["integer"].start_with?("0")
                    if is_imaginary
                        grammar.imaginary_literal

                    elsif is_float
                        grammar.float_literal

                    elsif is_octal
                        if match[:integer] =~ /[89]/
                            raise illegal_octal_digit(SourceRange.new(syntax_tree, start_index, index))
                        end
                        grammar.octal_literal

                    elsif match[:integer]
                        grammar.integer_literal

                    else
                        raise internal_error(SourceRange.new(syntax_tree, start_index, index), "ERROR: number that doesn't fit any category.")
                    end
                end
            end

            def operators_regexp
                @operators_regexp ||= Regexp.new(
                    "\\A(" +
                    grammar.tokens.keys.select { |key| key.is_a?(String) }
                                      .sort_by { |key| -key.length }
                                      .map { |key| Regexp.escape(key) }
                                      .join("|") +
                    ")"
                )
            end

        end
    end
end