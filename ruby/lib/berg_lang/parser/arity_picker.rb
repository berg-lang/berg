module BergLang
    class Parser
        class ArityPicker
            attr_reader :source
            attr_reader :unclosed_expression

            def initialize(unclosed_expression)
                @source = unclosed_expression.source
                @unclosed_expression = unclosed_expression
            end

            def pick_infix(operators)
                # 1. Sticky Postfix: treat all guaranteed postfix operators as postfix 
                last_postfix = last_sticky_postfix(operators)

                # 2. Prefix > Postfix: Get as many prefix operators in a row as you can. The rest is postfix.
                first_prefix = first_possible_prefix(operators, last_postfix+1)
                last_postfix = first_prefix - 1
                last_postfix -= 1 if operators[last_postfix].is_a?(Whitespace)

                #
                # 3. Infix > Postfix: Use the last POSTFIX operator as INFIX if it can be.
                #
                if last_postfix >= 0 && operators[last_postfix].infix && !sticky_postfix?(operators, last_postfix)
                    infix = operators[last_postfix]
                    last_postfix -= 1 unless last_postfix < 0
                    last_postfix -= 1 if operators[last_postfix].is_a?(Whitespace)

                #
                # 4. Infix > Prefix: Use the first PREFIX operator as INFIX if it can be.
                #
                elsif first_prefix < operators.size && operators[first_prefix].infix && !sticky_prefix?(operators, first_prefix)
                    infix = operators[first_prefix]
                    first_prefix += 1 unless first_prefix >= operators.size
                    first_prefix += 1 if operators[first_prefix].is_a?(Whitespace)

                #
                # 4. If not, see if there is a LINEBREAK between the "chosen infix" and the first_prefix.
                #
                elsif operators[last_postfix+1].is_a?(Whitespace) && operators[last_postfix+1].has_newline?
                    infix = Operator.new(operators[last_postfix+1].match, unclosed_expression.all_operators["\n"])

                #
                # 5. Insert a CALL operator.
                #
                else
                    if operators[first_prefix-1].is_a?(Whitespace)
                        call_range = operators[first_prefix-1].source_range
                    elsif operators[first_prefix]
                        call_range = source.create_empty_range(operators[first_prefix].begin)
                    elsif operators[last_postfix]
                        call_range = source.create_empty_range(operators[last_postfix].end)
                    else
                        call_range = source.create_empty_range(unclosed_expression.source_range.begin)
                    end
                    infix = Operator.new(call_range, unclosed_expression.all_operators[:call])
                end

                [ last_postfix, infix, first_prefix ]
            end

            private

            def last_sticky_postfix(operators)
                operators.each_with_index do |operator, index|
                    return index-1 if !(operator.is_a?(Operator) && operator.postfix)
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
                if operators[index].is_a?(Operator) && operators[index].postfix
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
                if operators[index].is_a?(Operator) && operators[index].prefix
                    prev_operator = operators[index-1] if index >= 0
                    prev_operator.is_a?(Whitespace) && !operators[index+1].is_a?(Whitespace)
                end
            end
        end
    end
end