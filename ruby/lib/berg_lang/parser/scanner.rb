require_relative "syntax_errors"

module BergLang
    class Parser
        #
        # Scans for tokens from the stream (whitespace, numbers, strings, words).
        #
        class Scanner
            attr_reader :parser
            attr_reader :stream

            def initialize(parser)
                @parser = parser
                @stream = source.open
                @state = :not_started
            end

            def index
                stream.index
            end

            def read_token
                case state
                when :not_started
                    @state = :parsing
                    return parser.tokens.sof

                when :parsing
                    if stream.eof?
                        @state = :eof
                        return parser.tokens.eof
                    end

                    token = parse_whitespace || parse_number || parse_operator || parse_string || parse_bareword
                    if !token
                        raise syntax_errors.unrecognized_character(create_token(stream.match(/./), parser.tokens.unrecognized_character))
                    end
                    token

                else
                    nil
                end
            end

            private

            attr_reader :state

            def syntax_errors
                SyntaxErrors.new
            end

            def parse_space
                if stream.match(/\A\r?\n/)
                    parser.tokens.newline
                elsif stream.match(/\A[[:blank:]]+/)
                    parser.tokens.whitespace
                elsif stream.match(/\A#[^\n]*/)
                    parser.tokens.comment
                end
            end

            def parse_operator
                if stream.match(operators_regexp)
                    parser.tokens.operators[match]
                end
            end

            def parse_bareword
                if stream.match(/\A(\w|[_$])+/)
                    parser.tokens.bareword
                end
            end

            def parse_string
                if stream.peek == '"'
                    if stream.match(/\A"(\\.|[^\\"])*"/m)
                        parser.tokens.string_literal
                    else
                        raise syntax_errors.unclosed_string(stream.match(/\A"([\\.|[^\\"\n])/m))
                    end
                end
            end

            def parse_number
                # Handle hex literals (0xDEADBEEF)
                # prefix integer
                if stream.match(/\A(?<prefix>0[xX])(?<integer>(\d|[A-Fa-f])+)/)
                    illegal_word_characters = stream.match /\A(\w|[_$])+/
                    # Word characters immediately following a number is illegal.
                    if illegal_word_characters
                        raise syntax_errors.variable_name_starting_with_an_integer(SourceRange.span(match, illegal_word_characters))
                    end
                    return parser.tokens.hexadecimal_literal
                end

                #
                # Handle floats, imaginaries and integers (hex is later in this function)
                #
                # integer (. decimal)? (e expsign? exponent)? i?
                if match = stream.match /\A(?<integer>\d+)((\.)(?<decimal>\d+))?((e)(?<expsign>[-+])?(?<exp>\d+))?(?<imaginary>i)?/i
                    illegal_word_characters = stream.match /\A(\w|[_$])+/
                    # Word characters immediately following a number is illegal.
                    if illegal_word_characters
                        if !match[:exp] && !match[:imaginary] && illegal_word_characters.string.downcase == "e"
                            raise syntax_errors.empty_exponent(SourceRange.span(match, illegal_word_characters))
                        elsif match[:decimal]
                            raise syntax_errors.float_with_trailing_identifier(SourceRange.span(match, illegal_word_characters))
                        else
                            raise syntax_errors.variable_name_starting_with_an_integer(SourceRange.span(match, illegal_word_characters))
                        end
                    end

                    is_imaginary = match[:imaginary]
                    is_float = match[:decimal] || match[:exp]
                    is_octal = !is_float && match[:integer] && match[:integer].length > 1 && match["integer"].start_with?("0")
                    if is_imaginary
                        parser.tokens.imaginary_literal

                    elsif is_float
                        parser.tokens.float_literal

                    elsif is_octal
                        if match[:integer] =~ /[89]/
                            raise syntax_errors.illegal_octal_digit(match)
                        end
                        parser.tokens.octal_literal

                    elsif match[:integer]
                        parser.tokens.integer_literal

                    else
                        raise syntax_errors.internal_error(match, "ERROR: number that doesn't fit any category: #{match.string}")
                    end
                end
            end

            def operators_regexp
                @operators_regexp ||= Regexp.new(
                    "\\A(" +
                    parser.tokens.operators.keys.select { |key| key.is_a?(String) }
                                      .sort_by { |key| -key.length }
                                      .map { |key| Regexp.escape(key) }
                                      .join("|") +
                    ")"
                )
            end

        end
    end
end