require_relative "parser/arity_picker"
require_relative "parser/operator"
require_relative "parser/output"
require_relative "parser/tokenizer"
require_relative "parser/unclosed_expression"
require_relative "parser/syntax_errors"
require_relative "parser/indent_operator"
require_relative "expressions/empty_expression"

module BergLang
    #
    # Parses Berg.
    #
    class Parser
        attr_reader :source
        attr_reader :output

        def initialize(source, output: Output.new(STDOUT))
            @source = source
            @output = output
            @tokenizer = Tokenizer.new(source, output)
            @token = tokenizer.advance_token
            @unclosed_expression = UnclosedExpression.new(self)
            @current_indent = source.create_empty_range(0)
        end

        def all_operators
            tokenizer.all_operators
        end

        def parse
            # Prefix <sof> PREFIX* E
            operators, expression = next_expression_phrase
            unclosed_expression.apply_prefix!(operators)
            unclosed_expression.apply_expression!(expression) if expression

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
        attr_reader :current_indent

        # TODO handle this in the tokenizer.
        def next_expression_phrase
            operators = []
            while true
                output.debug "Token: #{token.to_s.inspect}"
                case token
                when Expression
                    return [operators, advance_token]

                when Operator
                    if empty_expression = handle_empty_block(token, operators)
                        return [operators, empty_expression]
                    end

                    # Grab the indent before advancing token (in case we need it)
                    indent = current_indent
                    operator = token
                    operators << advance_token

                    handle_indent(operator, operators, indent)

                when Whitespace
                    # If there's a newline, handle the indent level.
                    if token.newline
                        # Save the current level of indent in case we see a new declaration on this line.
                        @current_indent = token.indent

                        if empty_expression = handle_undent(token, operators)
                            return [operators, empty_expression]
                        end
                    end

                    operators << advance_token

                when nil
                    return [operators, nil]
                else
                    raise syntax_errors.internal_error(token, "Unknown token type #{token.class}")
                end
            end
        end

        def handle_empty_block(operator, operators)
            # Anything that opens up an expression block can have an empty expression in it: (, { and :, for example.
            # Empty expressions aren't neatly handled by our [operators, expression] abstraction because it's essentially
            # an expression hidden in the operators; so we insert them here and terminate the [operators, expression]
            # block early with an empty expression when they happen.
            if operator.close
                #
                # Explicit end:
                #   ()
                #   {}.
                #
                # Infix blocks: empty infix blocks can happen if a ) or EOF happens on the same line as the open:
                #
                #   Decl: <eof>
                #   (A: 1, B: 2, Decl: )
                #
                # Empty infix blocks can also happen if the next line is indented more, but that is handled by the "explicit end" case
                # above.
                case operators[-1]
                when Operator
                    prev_operator = operators[-1]
                when Whitespace
                    if operators[-2].is_a?(Operator)
                        prev_operator = operators[-2]
                    end
                end
                if prev_operator && (prev_operator.open || prev_operator.opens_indent_block?)
                    output.debug("Empty Block: operation #{operator} #{prev_operator} found. Inserting empty expression between.")
                    # The empty range is from the end of the starting operator (just after :, ( or { ) to the current position,
                    # so it includes any whitespace.
                    empty_range = SourceRange.new(source, prev_operator.source_range.end, operators[-1].source_range.end)
                    # Don't advance the token, so that next time we come around after the empty expression, it will still pick up the )
                    Expressions::EmptyExpression.new(empty_range)
                end
            end
        end

        def handle_indent(operator, operators, indent)
            #
            # Handle open indent: if we see a : operator followed by \n, insert an open indent before the whitespace comes around.
            # possible for the next expression to have a *smaller* indent, in which case an undent and empty expression will happen.
            #
            if operator.opens_indent_block? && token.is_a?(Whitespace) && token.newline
                output.debug("Indent: #{operator} followed by newline. Current indent is #{indent.string.inspect}.")
                indent_start = source.create_empty_range(operator.source_range.end)
                open_indent = IndentOperator.new(indent_start, indent, all_operators[:indent])
                operators << open_indent
            end
        end

        def handle_undent(whitespace, operators)
            open_indents = unclosed_expression.open_indents
            open_indents = open_indents + operators.select { |operator| operator.is_a?(IndentOperator) }
            open_indents.reverse_each do |open_indent|
                # Truncate both indents and make sure they match as far as tabs/spaces go
                if open_indent.indent.string[0...whitespace.indent.size] != whitespace.indent.string[0...open_indent.indent.size]
                    raise syntax_errors.unmatchable_indent(whitespace.indent, open_indent.indent)
                end

                # If we're properly indented, we won't find any further smaller indents. Exit early.
                break if whitespace.indent.size > open_indent.indent.size

                output.debug("Undent: #{whitespace.indent.string.inspect} followed by newline")
                undent = Operator.new(source.create_empty_range(whitespace.indent.end), all_operators[:undent])
                empty_expression = handle_empty_block(undent, operators)
                return empty_expression if empty_expression

                operators << undent
            end
            nil
        end

        def advance_token
            # We do this dance so we can essentially look at *three* tokens at once. TODO do that buffering in the tokenizer.
            previous_token = token
            @token = tokenizer.advance_token
            previous_token
        end

        def syntax_errors
            SyntaxErrors.new
        end
    end
end
