require_relative "parser/arity_picker"
require_relative "parser/operator"
require_relative "parser/tokenizer"
require_relative "parser/unclosed_expression"

module Berg
    #
    # Parses Berg.
    #
    class Parser
        attr_reader :source

        def initialize(source)
            @source = source
            @tokenizer = Tokenizer.new(source)
            @token = tokenizer.advance_token
            puts "SET TOKEN #{token}"
            @unclosed_expression = UnclosedExpression.new
        end

        def parse
            # Prefix <sof> PREFIX* E
            operators, expression = next_expression_phrase
                puts "First: Operators: #{operators.inspect}, Expression: #{expression}"
            unclosed_expression.apply_prefix!(operators)
            unclosed_expression.apply_expression!(expression)

            loop do
                operators, expression = next_expression_phrase
                puts "Next: Operators: #{operators.join(", ")}, Expression: #{expression}"

                # Infix (E POSTFIX* INFIX PREFIX* E)
                if expression
                    unclosed_expression.resolve_infix!(operators, tokenizer.operator_list)
                    unclosed_expression.apply_expression!(expression)

                # Postfix (E POSTFIX* <eof>)
                else
                    unclosed_expression.apply_postfix!(operators)
                    return unclosed_expression.expression
                end

            end
        end

        private

        attr_reader :tokenizer
        attr_reader :unclosed_expression
        attr_reader :token

        def next_expression_phrase
            operators = []
            while token && !token.is_a?(Expression)
                operators << advance_token
            end
            expression = advance_token
            [operators, expression]
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
                        # TODO line numbers and all that jazz
                        raise "Indents do not match due to tabs and spaces!"
                    end
                    next_token = Operator.new(next_token.match, tokenizer.operator_list[:undent])
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
