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

module Berg
    class Parser
        #
        # Parses Berg.
        #
        class Tokenizer
            attr_reader :source

            def initialize(source)
                @source = source
                @token = Operator.new(source.match(//), operator_list[:sof])
            end

            def token
                if @token == :next
                    @token = parse_whitespace
                    @token ||= parse_operator
                    @token ||= parse_string
                    @token ||= parse_number
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

            def operator_list
                OperatorList.berg_operators
            end

            private

            def syntax_errors
                SyntaxErrors.new(source)
            end

            def parse_whitespace
                match = source.match(/^((((?<newline>\n)(?<indent>[ \t]*))|\s)+|#[^\n]+)+/)
                Whitespace.new(match) if match
            end

            def parse_operator
                match = source.match(operators_regex)
                Operator.new(match, operator_list[match.to_s]) if match
            end

            def parse_bareword
                match = source.match(/^(\w|[_$])+/)
                Expressions::Bareword.new(match) if match
            end

            def parse_string
                match = source.match(/^"(\\.|[^\\"]+)*"/)
                Expressions::StringLiteral.new(match) if match
            end

            def eof_token
                @eof_token ||= Operator.new(source.match(//), operator_list[:eof])
            end

            def parse_number
                #
                # Handle floats, imaginaries and integers (hex is later in this function)
                #
                # sign? integer? (. decimal) (e expsign? exponent)? i?
                match = source.match /^(?<sign>[-+])?(?<integer>\d+)?((\.)(?<decimal>\d+))((e)(?<expsign>[-+])?(?<exp>\d+))?(?<imaginary>i)?/
                # sign? integer (. decimal)? (e expsign? exponent)? i?
                match ||= source.match /^(?<sign>[-+])?(?<integer>\d+)((\.)(?<decimal>\d+))?((e)(?<expsign>[-+])?(?<exp>\d+))?(?<imaginary>i)?/
                if match
                    is_float = match[:decimal] || match[:exp]
                    is_octal = !is_float && match[:integer] && match[:integer].length > 1 && match[:integer][0] == "0"
                    if match[:imaginary]
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
                        raise syntax_error.internal_error(match, "ERROR: number that doesn't fit any category: #{match}")
                    end
                else
                    # Handle hex literals (0xDEADBEEF)
                    # sign? prefix integer
                    match = source.match /^(?<sign>[-+])?(?<prefix>0x)(?<integer>(\d|[A-Fa-f])+)/
                    if match
                        Expressions::HexadecimalLiteral.new(match)
                    else
                        nil
                    end
                end
            end

            def operators_regex
                @operators_regex ||= Regexp.new("^(" +
                    operator_list.keys.select { |key| key.is_a?(String) }.sort_by { |key| -key.length }.map { |key| Regexp.escape(key) }.join("|") + ")"
                )
            end

        end
    end
end