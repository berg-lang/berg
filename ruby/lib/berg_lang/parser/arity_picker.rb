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
                puts "Operators: #{operators.map { |op| op.to_s.inspect }.join(", ")}"
                #
                # 1. Infix > Prefix > Postfix:
                #
                # Go through the list of operators and pick as many prefix operators as you can pick followed by an infix operator.
                # If you run into an operator that MUST be postfix or run out of operators, insert a call there.
                #
                first_prefix, infix_index, newline_index = pick_max_prefixes(operators)

                #
                # Normal infix
                #
                if infix_index
                    postfixes = operators[0...infix_index]
                    infix = operators[infix_index]
                    prefixes = operators[infix_index+1..-1]

                #
                # Newline
                #
                elsif newline_index
                    postfixes, infix, prefixes = insert_newline_operator(operators, newline_index)
                else
                    postfixes, infix, prefixes = insert_call_operator(operators, first_prefix)
                end

                puts "Postfixes: #{postfixes.map { |op| op.to_s.inspect }.join(", ")}"
                puts "Infix: #{infix.to_s.inspect}"
                puts "Prefixes: #{prefixes.map { |op| op.to_s.inspect }.join(", ")}"

                assert_postfix!(postfixes, operators)
                assert_prefix!(prefixes, operators)

                [ postfixes, infix, prefixes ]
            end

            def pick_max_prefixes(operators)
                index = operators.size-1
                while index >= 0
                    if operators[index].is_a?(Operator)
                        postfix, infix, prefix = possible_arities(operators, index)

                        infix_index = index if infix

                        if prefix
                            # If it MUST be prefix, we can't choose the infix we wanted to, because those must be prefix too.
                            if !infix && !postfix
                                infix_index = nil
                                newline_index = nil
                            end
                        else
                            break
                        end
                    elsif operators[index].newline
                        newline_index = index
                    end

                    index -= 1
                end
                [ index+1, infix_index, newline_index ]
            end

            def puts(string)
                output.debug string
            end

            def insert_newline_operator(operators, newline_index)
                puts "Newline Statement: Inserting newline separator."
                postfixes = operators[0...newline_index]
                # TODO split up the whitespace so that the infix lies in between the pr3efixes and postfixes
                infix = Operator.new(operators[newline_index].newline, unclosed_expression.all_operators["\n"])
                prefixes = operators[newline_index..-1]
                [ postfixes, infix, prefixes ]
            end

            def insert_call_operator(operators, first_prefix)
                puts "Call Expression: Inserting call operator at #{first_prefix}."

                # Pick the spot right after the left expression to be the call operator.
                call_location = (first_prefix > 0 ? operators[first_prefix-1] : unclosed_expression).source_range.end
                postfixes = operators[0...first_prefix]
                infix = Operator.new(source.create_empty_range(call_location), unclosed_expression.all_operators[:call])
                prefixes = operators[first_prefix...operators.size]
                [ postfixes, infix, prefixes ]
            end

            def assert_postfix!(postfixes, operators=postfixes)
                postfixes.each_with_index do |operator, index|
                    next unless operator.is_a?(Operator)

                    if !operator.postfix || operator.postfix.resolve_manually?
                        # If we are not postfix, we must have been forced to be so by the next operator.
                        next_operator = index+1
                        next_operator += 1 unless operators[next_operator].is_a?(Operator)
                        raise syntax_errors.internal_error(operator, "No infix/prefix operator before non-postfix operator #{operator}: can't figure out how we got an error!") if !operators[next_operator]
                        raise syntax_errors.missing_value_between_operators(operator, operators[next_operator])
                    end
                end
            end

            def assert_prefix!(prefixes, operators=prefixes)
                (operators.size-prefixes.size).upto(operators.size-1) do |index|
                    operator = operators[index]
                    next unless operator.is_a?(Operator)
                    if !operator.prefix || operator.prefix.resolve_manually?
                        previous_operator = index-1
                        previous_operator -= 1 unless operators[previous_operator].is_a?(Operator)
                        raise syntax_errors.internal_error(operator, "No infix/postfix operator before non-prefix operator #{operator}: can't figure out how we got an error!") if previous_operator < 0
                        raise syntax_errors.missing_value_between_operators(operators[previous_operator], operator)
                   end
                end
            end

            # Returns the actual possibilities for the given operator. Takes stickiness into account.
            def possible_arities(operators, index)
                operator = operators[index]
                postfix = operator.postfix unless operator.postfix && operator.postfix.resolve_manually?
                infix = operator.infix unless operator.infix && operator.infix.resolve_manually?
                prefix = operator.prefix unless operator.prefix && operator.prefix.resolve_manually?
                if sticky_prefix?(operators, index)
                    [ nil, nil, prefix ]
                elsif sticky_postfix?(operators, index)
                    [ postfix, nil, nil ]
                else
                    [ postfix, infix, prefix ]
                end
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
                    next_operator = operators[index+1]
                    prev_is_whitespace = prev_operator && (prev_operator.is_a?(Whitespace) || [:undent, :indent].include?(prev_operator.key))
                    next_is_whitespace = next_operator && (next_operator.is_a?(Whitespace) || [:undent, :indent].include?(next_operator.key))
                    if !prev_is_whitespace && next_is_whitespace
                        puts "Sticky Postfix: #{operators[index].to_s.inspect}"
                        true
                    end
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
                    prev_operator = operators[index-1] if index > 0
                    next_operator = operators[index+1]
                    prev_is_whitespace = prev_operator && (prev_operator.is_a?(Whitespace) || [:undent, :indent].include?(prev_operator.key))
                    next_is_whitespace = next_operator && (next_operator.is_a?(Whitespace) || [:undent, :indent].include?(next_operator.key))
                    if !next_is_whitespace && prev_is_whitespace
                        puts "Sticky Prefix: #{operators[index].to_s.inspect}"
                        true
                    end
                end
            end

            def syntax_errors
                SyntaxErrors.new
            end
        end
    end
end