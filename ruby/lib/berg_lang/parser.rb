require_relative "parser/arity_picker"
require_relative "parser/operator"
require_relative "parser/tokenizer"
require_relative "parser/unclosed_expression"
require_relative "parser/syntax_errors"
require_relative "expressions/empty_expression"

module BergLang
    #
    # Parses Berg.
    #
    class Parser
        attr_reader :source

        def initialize(source)
            @source = source
            @tokenizer = Tokenizer.new(source)
            @token = tokenizer.advance_token
            @unclosed_expression = UnclosedExpression.new(self)
        end

        def all_operators
            tokenizer.all_operators
        end

        def parse
            # Prefix <sof> PREFIX* E
            operators, expression = next_expression_phrase
            unclosed_expression.apply_prefix!(operators)
            unclosed_expression.apply_expression!(expression)

            loop do
                operators, expression = next_expression_phrase

                # Infix (E POSTFIX* INFIX PREFIX* E)
                if expression
                    unclosed_expression.resolve_infix!(operators)
                    unclosed_expression.apply_expression!(expression)

                # Postfix (E POSTFIX* <eof>)
                else
                    unclosed_expression.apply_postfix!(operators)
                    return unclosed_expression.expression
                end

            end
        end

        def source_range
            [ unclosed.first[0], unclosed.last[1] ]
        end

        private

        attr_reader :tokenizer
        attr_reader :unclosed_expression
        attr_reader :token

        def next_expression_phrase
            operators = []
            while true
                case token
                when Expression
                    return [operators, advance_token]
                when Operator
                    # Empty () ends up showing up as a series of tokens, but there is an assumed EmptyExpression in there
                    if token.end_delimiter
                        if operators[-1].is_a?(Operator) && operators[-1].start_delimiter
                            return [operators, Expressions::EmptyExpression.new(source.create_empty_range(operators[-1].source_range.end))]
                        elsif operators[-1].is_a?(Whitespace) && operators[-2].is_a?(Operator) && operators[-2].start_delimiter
                            return [operators, Expressions::EmptyExpression.new(operators[-1].source_range)]
                        end
                    end
                    operators << advance_token
                when Whitespace
                    operators << advance_token
                when nil
                    return [operators, nil]
                else
                    raise syntax_errors.internal_error(token, "Unknown token type #{token.class}")
                end
            end
        end

        def advance_token
            previous_token = token
            next_token = tokenizer.advance_token

            #
            # If we encounter a lower level of indent than current, generate :undent instead of whitespace.
            #
            if next_token.is_a?(Whitespace) && next_token.has_newline?
                current_indent = next_token.indent
                start_delimiter = unclosed_expression.current_start_delimiter
                if start_delimiter && current_indent.size >= start_delimiter.size
                    if !current_indent.start_with?(start_delimiter.match[:indent])
                        raise syntax_errors.unmatchable_indent(next_token, start_delimiter)
                    end
                    next_token = Operator.new(next_token.match, tokenizer.all_operators[:undent])
                end
            end

            @token = next_token
            previous_token
        end

        def syntax_error(message)
            raise message
        end
    end
end
