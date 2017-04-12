require_relative "whitespace"
require_relative "operator"
require_relative "operator_list"
require_relative "unrecognized_character"
require_relative "syntax_errors"
require_relative "../expressions/bareword"
require_relative "../expressions/string_literal"
require_relative "../expressions/imaginary_literal"
require_relative "../expressions/float_literal"
require_relative "../expressions/octal_literal"
require_relative "../expressions/hexadecimal_literal"
require_relative "../expressions/integer_literal"

module BergLang
    class Parser
        #
        # Parses Berg.
        #
        class Tokenizer
            attr_reader :source

            def initialize(source)
                @source = source
                @token = Operator.new(source.create_empty_range, all_operators[:sof])
            end

            def token
                if @token == :next
                    @token = parse_whitespace
                    @token ||= parse_number
                    @token ||= parse_operator
                    @token ||= parse_string
                    @token ||= parse_bareword
                    @token ||= eof_token if source.eof?
                    if !token
                        raise syntax_errors.unrecognized_character(UnrecognizedCharacter.new(source.match(/./)))
                    end
                    @token
                else
                    @token
                end
            end

            # Pick up the current token, and ensure we will pick up a new token next time.
            def advance_token
                result = self.token
                case result
                when eof_token
                    @token = nil
                when nil
                else
                    @token = :next
                end
                result
            end

            def all_operators
                OperatorList.berg_operators
            end

            private

            def syntax_errors
                SyntaxErrors.new
            end

            def parse_whitespace
                match = source.match(/\A((((?<newline>\n)(?<indent>[ \t]*))|\s)+|#[^\n]+)+/)
                Whitespace.new(match) if match
            end

            def parse_operator
                match = source.match(operators_regex)
                if match
                    if match.string == "." && digits = source.match(/\A\d+/)
                        raise syntax_errors.float_without_leading_zero(SourceRange.span(match, digits))
                    end
                    Operator.new(match, all_operators[match.string])
                end
            end

            def parse_bareword
                match = source.match(/\A(\w|[_$])+/)
                Expressions::Bareword.new(match) if match
            end

            def parse_string
                if source.peek == '"'
                    match = source.match(/\A"(\\.|[^\\"]+)*"/m)
                    if match
                        Expressions::StringLiteral.new(match)
                    else
                        match = source.match(/\A"(\\.|[^\\"]+)*/m)
                        raise syntax_errors.unclosed_string(match)
                    end
                end
            end

            def eof_token
                @eof_token ||= Operator.new(source.create_empty_range, all_operators[:eof])
            end

            def parse_number
                #
                # Handle floats, imaginaries and integers (hex is later in this function)
                #
                # integer (. decimal)? (e expsign? exponent)? i?
                match = source.match /\A(?<integer>\d+)((\.)(?<decimal>\d+))?((e)(?<expsign>[-+])?(?<exp>\d+))?(?<imaginary>i)?/i
                if match
                    illegal_word_characters = source.match /\A(\w|[_$])+/
                    # Word characters immediately following a number is illegal.
                    if illegal_word_characters
                        if !match[:exp] && illegal_word_characters.string.downcase == "e"
                            raise syntax_errors.empty_exponent(illegal_word_characters)
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
                        Expressions::ImaginaryLiteral.new(match)

                    elsif is_float
                        Expressions::FloatLiteral.new(match)

                    elsif is_octal
                        if match[:integer] =~ /[89]/
                            raise syntax_errors.illegal_octal_digit(match)
                        end
                        Expressions::OctalLiteral.new(match)

                    elsif match[:integer]
                        Expressions::IntegerLiteral.new(match)

                    else
                        raise syntax_errors.internal_error(match, "ERROR: number that doesn't fit any category: #{match.string}")
                    end
                else
                    # Handle hex literals (0xDEADBEEF)
                    # sign? prefix integer
                    match = source.match /\A(?<sign>[-+])?(?<prefix>0x)(?<integer>(\d|[A-Fa-f])+)/
                    if match
                        Expressions::HexadecimalLiteral.new(match)
                    else
                        nil
                    end
                end
            end

            def operators_regex
                @operators_regex ||= Regexp.new("^(" +
                    all_operators.keys.select { |key| key.is_a?(String) }.sort_by { |key| -key.length }.map { |key| Regexp.escape(key) }.join("|") + ")"
                )
            end

        end
    end
end