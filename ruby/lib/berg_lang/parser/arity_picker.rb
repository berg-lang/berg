require_relative "syntax_errors"

module BergLang
    class Parser
        class ArityPicker
            attr_reader :source
            attr_reader :output
            attr_reader :unclosed_expression

            def initialize(unclosed_expression)
                @source = unclosed_expression.source
                @output = unclosed_expression.parser.output
                @unclosed_expression = unclosed_expression
            end

            def pick_infix(operators)
                output.debug "Operators: #{operators.map { |op| op.to_s.inspect }.join(", ")}"
                #
                # 1. Infix > Prefix > Postfix:
                #
                # Go through the list of operators and pick as many prefix operators as you can pick followed by an infix operator.
                # If you run into an operator that MUST be postfix or run out of operators, insert a call there.
                # 
                index = operators.size-1
                while index >= 0
                    operator = operators[index]
                    if operator.is_a?(Operator)
                        # Break on the first postfix-only operator
                        break if sticky_postfix?(operators, index)
                        # Things that MUST be prefix are always OK.

                        if operator.infix
                            infix_index = index
                        end
                        # If we find something that MUST be prefix, we can't choose the infix we wanted to ...
                        if sticky_prefix?(operators, index) || (!operator.infix && !operator.postfix)
                            infix_index = nil
                            newline_index = nil
                        end
                        # If it doesn't allow prefix, we don't continue to the next one.
                        break unless operator.prefix

                    elsif operator.newline
                        newline_index = index
                    end

                    index -= 1
                end

                #
                # Normal infix
                #
                if infix_index
                    postfixes = operators[0...infix_index]
                    infix = operators[infix_index]
                    prefixes = operators[infix_index+1..-1]

                    # When there's a normal infix operator, we have a special (better) error message to give if the postfixes
                    # aren't really postfixes.
                    postfixes.each do |operator|
                        if operator.is_a?(Operator) && !operator.postfix
                            raise syntax_errors.prefix_or_infix_in_front_of_infix_operator(operator, infix)
                        end
                    end

                #
                # Newline
                #
                elsif newline_index
                    output.debug("Newline Statement: Inserting newline separator.")
                    postfixes = operators[0...newline_index]
                    # TODO split up the whitespace so that the infix lies in between the pr3efixes and postfixes
                    infix = Operator.new(operators[newline_index].newline, unclosed_expression.all_operators["\n"])
                    prefixes = operators[newline_index..-1]

                #
                # Call operator
                #
                else
                    output.debug("Call Expression: Inserting call operator.")

                    # Pick the spot right after the left expression to be the call operator.
                    call_location = (index >= 0 ? operators[index] : unclosed_expression).source_range.end
                    # Everything up to and including index
                    postfixes = operators[0...index+1]
                    infix = Operator.new(source.create_empty_range(call_location), unclosed_expression.all_operators[:call])
                    prefixes = operators[index+1..-1]
                end

                output.debug "Postfixes: #{postfixes.map { |op| op.to_s.inspect }.join(", ")}"
                output.debug "Infix: #{infix.to_s.inspect}"
                output.debug "Prefixes: #{prefixes.map { |op| op.to_s.inspect }.join(", ")}"

                [ postfixes, infix, prefixes ]
            end

            private

            def last_sticky_postfix(operators)
                operators.each_with_index do |operator, index|
                    return index-1 unless operator.is_a?(Operator) && operator.postfix && operator.postfix.can_be_sticky?
                end
                -1
            end

            def first_possible_prefix(operators, start_index)
                first_prefix = operators.size
                (operators.size-1).downto(start_index).each do |index|
                    operator = operators[index]
                    if operator.is_a?(Operator)
                        if operator.prefix
                            first_prefix = index
                        else
                            break
                        end
                    end
                end
                first_prefix
            end

            #
            # An operator with whitespace on the right and no whitespace on the left is "sticky postfix",
            # i.e. it will always be treated as postfix if it can be.
            #
            # For example, a+\nb is (a+) \n b (two separate statements), not a + b
            #
            def sticky_postfix?(operators, index)
                if operators[index].is_a?(Operator) && operators[index].postfix && operators[index].postfix.can_be_sticky?
                    prev_operator = operators[index-1] if index > 0
                    !prev_operator.is_a?(Whitespace) && operators[index+1].is_a?(Whitespace)
                end
            end

            #
            # An operator with whitespace on the lefty and no whitespace on the right is "sticky prefix",
            # i.e. it will always be treated as prefix if it can be.
            #
            # For example, a -b is a(-b), not a - b.
            #
            def sticky_prefix?(operators, index)
                if operators[index].is_a?(Operator) && operators[index].prefix && operators[index].prefix.can_be_sticky?
                    prev_operator = operators[index-1] if index >= 0
                    prev_operator.is_a?(Whitespace) && !operators[index+1].is_a?(Whitespace)
                end
            end

            def syntax_errors
                SyntaxErrors.new
            end
        end
    end
end