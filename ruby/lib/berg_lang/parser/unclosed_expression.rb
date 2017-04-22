require_relative "syntax_errors"
require_relative "../expressions/postfix_operation"
require_relative "../expressions/delimited_operation"
require_relative "../expressions/prefix_operation"
require_relative "../expressions/infix_operation"
require_relative "../expressions/empty_expression"

module BergLang
    class Parser
        #
        # Resolves "loose" operators and whitespace in an expression while we read more and more
        # bits from the source file. Because precedence prevents us from knowing *exactly* what will be
        # in the right hand side of an expression, we have to keep the operators around until we can
        # create the actual expression.
        #
        # It keeps the operators and expressions floating around in the "unclosed" array, in the order
        # they appeared in the source. When it can resolve an operator into an expression (when it knows
        # for sure what the left and/or right side of the expression will be), it removes the operators from
        # the array and inserts the expression in its place. Rinse and repeat until only one remains.
        #
        class UnclosedExpression
            attr_reader :source
            attr_reader :parser

            def initialize(parser)
                @source = parser.source
                @parser = parser
                @unclosed = []
                @arity_picker = ArityPicker.new(self)
            end

            def all_operators
                parser.all_operators
            end

            def source_range
                first = unclosed.first
                first = first[0] if first.is_a?(Array)
                last = unclosed.last
                last = last[0] if last.is_a?(Array)
                SourceRange.span(first, last)
            end

            def open_indents
                unclosed.select { |token, operator| token.is_a?(IndentOperator) }.map { |token, operator| token }
            end

            def expression
                if unclosed.size == 0
                    Expressions::EmptyExpression.new
                elsif unclosed.size == 1 && unclosed[0].is_a?(Expression)
                    unclosed[0]
                else
                    raise syntax_errors.internal_error(source_range, "Expression still unclosed")
                end
            end

            def token_to_s(token, operator)
                str = token.to_s
                if str =~ /\s/
                    str = str.inspect
                end
                str
            end

            def unclosed_to_s(index=0, indent: "  ")
                token, operator = unclosed[index]
                token_str = token_to_s(token, operator)

                result = ""
                if operator
                    if operator.prefix?
                        result << "#{indent} \\- #{token_str}\n"
                        result << unclosed_to_s(index+1, indent: "#{indent}   #{" "*token_str.size}")
                    else
                        raise "Unexpected operator type #{operator.type} for #{operator}!"
                    end
                elsif token
                    next_token, next_operator = unclosed[index+1]
                    if next_operator && next_operator.infix?
                        next_operator_str = token_to_s(next_token, next_operator)
                        result = ""
                        result << "#{indent} \\   #{" "*next_operator_str.size}/- #{token_str}\n"
                        result << "#{indent}  \\- #{next_operator_str}\n"
                        result << unclosed_to_s(index+2, indent: "#{indent}    #{" "*next_operator_str.size}")
                    else
                        result << "#{indent} \\- #{token_str}\n"
                    end
                end
                result
            end

            #
            # Takes a set of operators between two expressions, and decides which are prefix, infix and postfix.
            # Then applies them.
            #
            def resolve_infix!(operators)
                postfixes, infix, prefixes = arity_picker.pick_infix(operators)
                apply_postfix!(postfixes)
                apply_infix!(infix)
                apply_prefix!(prefixes)
            end

            def apply_postfix!(operators)
                arity_picker.assert_postfix!(operators)
                operators.each do |operator|
                    next if operator.is_a?(Whitespace)

                    debug ""
                    debug "Postfix: #{token_to_s(operator, operator.postfix)}"
                    left_bind!(operator, operator.postfix) 
                    debug unclosed_to_s(indent: "  ")
                end
            end

            def apply_infix!(infix)
                debug ""
                debug "Infix: #{token_to_s(infix, infix.infix)}"
                left_bind!(infix, infix.infix)
                debug unclosed_to_s(indent: "  ")
            end

            def apply_prefix!(operators)
                arity_picker.assert_prefix!(operators)
                operators.each do |operator|
                    next if operator.is_a?(Whitespace)
                    debug "Applying Prefix Operator: #{token_to_s(operator, operator.prefix)}"
                    @unclosed << [ operator, operator.prefix ]
                    debug unclosed_to_s(indent: "  ")
                end
            end

            def apply_expression!(expression)
                debug ""
                debug "Expression: #{token_to_s(expression, nil)}"
                @unclosed << expression
                debug unclosed_to_s(indent: "  ")
            end

            private

            attr_reader :arity_picker
            attr_reader :unclosed

            def syntax_errors
                SyntaxErrors.new
            end

            def debug(string)
                parser.output.debug(string)
            end

            # PRE( PRE( (expr IN PRE( PRE( expr <- POST|IN
            def left_bind!(token, operator)
                # If it's an end delimiter, look for the corresponding start delimiter.
                if operator.close?
                    close_delimited!(token, operator)
                    return

                # It's postfix or infix. Take as many of the operators as we can (from right to left) as left children.
                else
                    left_child = nil
                    index = unclosed.size
                    while index > 0
                        next_index = index-1
                        left_token, left_operator = unclosed[next_index]
                        if !left_operator && next_index > 0
                            next_index -= 1
                            left_token, left_operator = unclosed[next_index]
                        end
                        if left_operator
                            # If the next operator can't have the left child, use the current one.
                            if !operator.can_have_left_child?(left_operator)
                                # Close the entire left child so we can dump it into the postfix/infix operator.
                                left_child = close!(index, token)
                                break
                            end
                        end
                        index = next_index
                    end

                    # No operator is willing to be a left child. Take the expression to the left instead.
                    left_child ||= close!(unclosed.size-1, token)
                    if operator.postfix?
                        unclosed << Expressions::PostfixOperation.new(left_child, token)
                    else
                        unclosed << left_child
                        unclosed << [ token, operator ]
                    end
                end
            end

            def close_delimited!(token, operator)
                # This loop will pass over other end delimiters (like in "(blah + { x: y )"), which will emit
                # an error in close! because they have no end delimiter.
                (unclosed.size-1).downto(0).each do |index|
                    left_token, left_operator = unclosed[index]
                    if left_operator
                        if operator.started_by == left_operator.key
                            # remove the open (
                            expression = close!(index+1, token)
                            unclosed.pop
                            unclosed << Expressions::DelimitedOperation.new(left_token, expression, token)
                            return
                        elsif left_operator.key == :indent
                            # Indents can be closed by any end delimiter
                            expression = close!(index+1, token)
                            unclosed.pop
                            unclosed << Expressions::DelimitedOperation.new(left_token, expression, token)
                        end
                    end
                end

                raise syntax_errors.unmatched_close(token)
            end

            #
            # Close off all unclosed expressions/operators from index on. This is used when:
            # - A postfix or infix operator is found, closing off the left hand side of the expression
            # - A close delimiter is found, closing off anything inside it
            #
            def close!(index, because_of)
                result, closed_index = close(index, because_of)
                @unclosed = unclosed[0...closed_index]
                result
            end

            def close(index, because_of)
                token, operator = unclosed[index]
                if operator
                    right_hand_side, closed_index = close(index+1, because_of)
                    if operator.close? && operator.key != :indent
                        # Explicit open operators (i.e. things other than indent) require explicit closes.
                        raise syntax_errors.unmatched_close(token, because_of)
                    elsif !right_hand_side
                        raise syntax_errors.missing_value_between_operators(token, because_of)
                    elsif operator.prefix?
                        expression = Expressions::PrefixOperation.new(token, right_hand_side)
                        [ expression, index ]
                    else
                        left_hand_side = unclosed[index-1]
                        expression = Expressions::InfixOperation.new(left_hand_side, token, right_hand_side)
                        [ expression, index-1 ]
                    end

                # If this is an expression and there is stuff to the right, close the right.
                elsif index+1 < unclosed.size
                    close(index+1, because_of)
                else
                    [ token, index ]
                end
            end
        end
    end
end
