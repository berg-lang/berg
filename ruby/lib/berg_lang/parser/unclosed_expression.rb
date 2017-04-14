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
                    raise syntax_errors.internal_error(source_range, "Expression still unclosed")
                end
            end

            def unclosed_to_s
                unclosed.map { |token, op| token }.join(", ")
            end

            #
            # Takes a set of operators between two expressions, and decides which are prefix, infix and postfix.
            # Then applies them.
            #
            def resolve_infix!(operators)
                last_postfix, operator, first_prefix = arity_picker.pick_infix(operators)
                apply_postfix!(operators[0..last_postfix], operator) if last_postfix >= 0
                apply_infix!(operator)

                # Indent (  A:\n    B) If it's indentable (a:\n...) and immediately followed by a linebreak, it's an indented value.
                if operator.infix.indentable?
                    whitespace = operators[first_prefix - 1]
                    if whitespace.is_a?(Whitespace) && whitespace.has_newline?
                        apply_prefix!([Operator.new(whitespace.indent, parser.all_operators[:indent])])
                    end
                end

                apply_prefix!(operators[first_prefix..-1])
            end

            def apply_prefix!(prefixes)
                prefixes.each do |operator|
                    debug "Prefix: #{operator}"
                    next if operator.is_a?(Whitespace)
                    if !operator.prefix
                        # If there is an empty expression--(<whitespace>)--it may show up in prefixes.
                        if operator.end_delimiter
                            close_delimited!(operator, operator.end_delimiter)
                        else
                            raise syntax_errors.missing_left_hand_side_at_sof(operator, prefixes[0])
                        end
                    end
                    @unclosed << [ operator, operator.prefix ]
                end

                debug "  - after: #{unclosed_to_s}" if unclosed.any?
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

            def apply_postfix!(postfixes, because_of_infix=nil)
                postfixes.each do |operator|
                    next if operator.is_a?(Whitespace)
                    debug "Postfix: #{operator}"
                    debug "  - before: #{unclosed_to_s}"
                    if !operator.postfix
                        if because_of_infix
                            raise syntax_errors.prefix_or_infix_in_front_of_infix_operator(operator, because_of_infix)
                        else
                            raise syntax_errors.missing_right_hand_side(operator, postfixes[-1])
                        end
                    end
                    left_bind!(operator, operator.postfix) 
                    debug "  - after:  #{unclosed_to_s}"
                end
            end

            private

            attr_reader :arity_picker
            attr_reader :unclosed

            def syntax_errors
                SyntaxErrors.new
            end

            def debug(string)
                # puts string
            end

            # PRE( PRE( (expr IN PRE( PRE( expr <- POST|IN
            def left_bind!(token, operator)
                # If it's an end delimiter, look for the corresponding start delimiter.
                if operator.end_delimiter?
                    close_delimited!(token, operator)
                    return

                # It's postfix or infix. Find the first expression (left to right) that can be a left child
                # of the POSTFIX or INFIX operator.
                else
                    left_child = nil
                    unclosed.each_with_index do |(left_token, left_operator), index|
                        next unless left_operator
                        if operator.can_have_left_child?(left_operator)
                            # Close the entire left child so we can dump it into the postfix/infix operator.
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
                    if operator.start_delimiter?
                        # Explicit open operators (i.e. things other than indent) require explicit closes.
                        raise syntax_errors.unmatched_start_delimiter(token, because_of)
                    elsif !right_hand_side
                        raise syntax_errors.missing_right_hand_side(token, because_of)
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
