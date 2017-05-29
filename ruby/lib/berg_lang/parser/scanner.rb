require_relative "syntax_errors"

module BergLang
    class Parser
        #
        # Scans for tokens from the stream (whitespace, numbers, strings, words).
        #
        class Scanner
            attr_reader :terms
            attr_reader :source
            attr_reader :stream
            attr_reader :index

            def initialize(parser)
                @source = parser.source
                @terms = parser.terms
                @stream = source.open
                @next_token = terms.sof
                @index = 0
                @eof = false
            end

            def eof?
                @eof && !@next_token
            end

            def peek
                @next_token
            end

            def peek_index
                stream.index
            end

            def next
                token = @next_token
                @index = stream.index
                @next_token = scan_token
                return token
            end

            private

            def scan_token
                # We're done when we've already 
                if stream.eof?
                    return nil if @eof
                    @eof = true
                    return terms.eof
                end

                token = parse_filler || parse_number || parse_operator || parse_string || parse_bareword
                if !token
                    raise unrecognized_character(create_token(stream.next, terms.unrecognized_character))
                end
                token
            end

            include SyntaxErrors

            def parse_filler
                if stream.match(/\A\r?\n/)
                    terms.newline
                elsif stream.match(/\A[[:blank:]]+/)
                    terms.whitespace
                elsif stream.match(/\A#[^\n]*/)
                    terms.comment
                end
            end

            def parse_operator
                if match = stream.match(operators_regexp)
                    terms.operators[match.to_s]
                end
            end

            def parse_bareword
                if stream.match(/\A(\w|[_$])+/)
                    terms.bareword
                end
            end

            def parse_string
                if stream.peek == '"'
                    if stream.match(/\A"(\\.|[^\\"])*"/m)
                        terms.string_literal
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
                    return terms.hexadecimal_literal
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
                        terms.imaginary_literal

                    elsif is_float
                        terms.float_literal

                    elsif is_octal
                        if match[:integer] =~ /[89]/
                            raise illegal_octal_digit(SourceRange.new(syntax_tree, start_index, index))
                        end
                        terms.octal_literal

                    elsif match[:integer]
                        terms.integer_literal

                    else
                        raise internal_error(SourceRange.new(syntax_tree, start_index, index), "ERROR: number that doesn't fit any category.")
                    end
                end
            end

            def operators_regexp
                @operators_regexp ||= Regexp.new(
                    "\\A(" +
                    terms.operators.keys.select { |key| key.is_a?(String) }
                                      .sort_by { |key| -key.length }
                                      .map { |key| Regexp.escape(key) }
                                      .join("|") +
                    ")"
                )
            end

        end
    end
end