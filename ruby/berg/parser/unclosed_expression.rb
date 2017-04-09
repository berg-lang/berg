require_relative "syntax_errors"
require_relative "../expressions/postfix_operation"
require_relative "../expressions/delimited_operation"
require_relative "../expressions/prefix_operation"
require_relative "../expressions/infix_operation"
require_relative "../expressions/empty_expression"

module Berg
    class Parser
        #
        # Handles "closing" an expression, resolving precedence and associativity rules for operators
        # and creating expressions for them.
        #
        class UnclosedExpression
            attr_reader :source

            def initialize(source)
                @source = source
                @unclosed = []
            end

            def current_start_delimiter
                unclosed.reverse_each do |operator|
                    return operator if operator.is_a?(Operator) && operator.prefix && operator.prefix.start_delimiter?
                end
                nil
            end

            def expression
                if unclosed.size == 0
                    Expressions::EmptyExpression.new
                elsif unclosed.size == 1 && unclosed[0].is_a?(Expression)
                    unclosed[0]
                else
                    raise syntax_errors.internal_error(unclosed[-1], "Expression still unclosed")
                end
            end

            def unclosed_to_s
                unclosed.map { |token, op| token }.join(", ")
            end

            #
            # Takes a set of operators between two expressions, and decides which are prefix, infix and postfix.
            # Then applies them.
            #
            def resolve_infix!(operators, operator_list)
                last_postfix, operator, first_prefix = ArityPicker.new.pick_infix(operators, operator_list)
                apply_postfix!(operators[0..last_postfix]) if last_postfix >= 0
                apply_infix!(operator)

                # Indent (  A:\n    B) If it's indentable (a:\n...) and immediately followed by a linebreak, it's an indented value.
                if operator.infix.indentable?
                    whitespace = operators[first_prefix - 1]
                    if whitespace.is_a?(Whitespace) && whitespace.has_newline?
                        apply_prefix!(Operator.new(whitespace.match, tokenizer.operator_list[:indent]))
                    end
                end

                apply_prefix!(operators[first_prefix..-1])
            end

            def apply_prefix!(prefixes)
                prefixes.each { |prefix| debug "Prefix: #{prefix}" }
                @unclosed += prefixes.select { |op| op.is_a?(Operator) }.map { |op| [ op, op.prefix ] }
                debug "  - after: #{unclosed_to_s}" if prefixes.any?
            end

            def apply_expression!(expression)
                debug "Expression: #{expression}"
                @unclosed << expression
                debug "  - after: #{unclosed_to_s}"
            end

            def apply_infix!(infix)
                debug "Infix: #{infix}"
                debug "  - before: #{unclosed_to_s}"
                left_bind!(infix, infix.infix)
                debug "  - after:  #{unclosed_to_s}"
            end

            def apply_postfix!(postfixes)
                postfixes.each do |operator|
                    next if operator.is_a?(Whitespace)
                    debug "Postfix: #{operator}"
                    debug "  - before: #{unclosed_to_s}"
                    left_bind!(operator, operator.postfix) 
                    debug "  - after:  #{unclosed_to_s}"
                end
            end

            private

            def syntax_errors
                SyntaxErrors.new(source)
            end

            def debug(string)
                # puts string
            end

            attr_reader :unclosed

            # PRE( PRE( (expr IN PRE( PRE( expr <- POST|IN
            def left_bind!(token, operator)
                # If it's an end delimiter, look for the corresponding start delimiter.
                if operator.end_delimiter?
                    close_delimited!(token, operator)
                    return

                # It's postfix or infix. Find the first expression (left to right) that can be a left child of the POSTFIX or INFIX operator.
                else
                    left_child = nil
                    unclosed.each_with_index do |(left_token, left_operator), index|
                        next unless left_operator
                        if operator.can_have_left_child?(left_operator)
                            # 2. Close the expression from that point on.
                            left_child = close!(index, token)
                            break
                        end
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

                raise syntax_errors.unmatched_end_delimiter(token)
            end

            def close!(index, because_of)
                result, closed_index = close(index, because_of)
                @unclosed = unclosed[0...closed_index]
                result
            end

            def close(index, because_of)
                token, operator = unclosed[index]
                if operator
                    right_hand_side, closed_index = close(index+1, because_of)
                    if operator.start_delimiter?
                        # Explicit open operators (i.e. things other than indent) require explicit closes.
                        raise syntax_errors.unmatched_start_delimiter(token, because_of)
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
