require_relative "../term_type"

module BergLang
    class Parser
        class TermType
            class Ambiguous < TermType
                attr_reader :expression
                attr_reader :infix
                attr_reader :prefix
                attr_reader :postfix
                attr_reader :filler

                def initialize(expression: nil, infix: nil, prefix: nil, postfix: nil, filler: nil)
                    @expression = expression
                    @infix = infix
                    @prefix = prefix
                    @postfix = postfix
                    @filler = filler
                    cache_best_variants!
                    validate!
                end

                def filler?
                    filler
                end
                def expression?
                    expression
                end
                def infix?
                    infix
                end
                def prefix?
                    prefix
                end
                def postfix?
                    postfix
                end

                def name
                    (expression || infix || prefix || postfix || filler).name
                end

                def string
                    (expression || infix || prefix || postfix || filler).string
                end

                #
                # Determine the best variant when the next operand is an operator, and when
                # it is an operand.
                #
                # @return [Boolean,Boolean] The preferred variant for operand and for operator
                # @example if_operand, if_operator = type.best_variants(true, true)
                #
                def preferred_variants(left, left_inserts_empty)
                    variant_index = 0
                    variant_index += PREFER_LEFT_OPERAND if left
                    variant_index += LEFT_INSERTS_EMPTY if left_inserts_empty
                    best_variant_cache[variant_index] ||= begin
                        if_operand, if_operator = [ false, true ].map do |right|
                            inserts_none  = term( left,  right)
                            inserts_left  = term(!left,  right)
                            inserts_right = term( left, !right)
                            inserts_both  = term(!left, !right)
                            if left == right
                                inserts_left ||= filler
                            else
                                inserts_none ||= filler
                            end

                            # If the right side inserts apply and the left inserts empty,
                            # we prefer to insert on the right. Otherwise, we always prefer
                            # to insert on the left (earliest wins, and apply > empty).
                            if left_inserts_empty && !right
                                inserts_none || inserts_right || inserts_left || inserts_both
                            else
                                inserts_none || inserts_left || inserts_right || inserts_both
                            end
                        end

                        # If the right sides are the same, we just pick the one that is
                        # preferred--it can't possibly influence the next term.
                        if_operand_next  =  if_operand.filler?  || !if_operand.right_is_operand?
                        if_operator_next = !if_operator.filler? && !if_operator.right_is_operand?
                        if if_operand_next == if_operator_next
                            if_operator = if_operand = left ? if_operand : if_operator
                        end
                        [ if_operand, if_operator ]
                    end
                end

                private

                PREFER_LEFT_OPERAND = 1
                LEFT_INSERTS_EMPTY = 2

                attr_reader :best_variant_cache

                def cache_best_variants!
                    @best_variant_cache = []
                    all_variants = variants.to_set
                    [ false, true ].each do |left|
                        [ false, true ].each do |left_inserts_empty|
                            operand, operator = preferred_variants(left, left_inserts_empty)
                            # If we return two different variants that have identical left and right sides,
                            # that is an error.
                            if operand != operator
                                operand_left   =  operand.filler?  || operand.left_is_operand?
                                operator_left  = !operator.filler? && operator.left_is_operand?
                                operand_right  =  operand.filler?  || !operand.right_is_operand?
                                operator_right = !operator.filler? && !operator.right_is_operand?
                                if operand_left == operator_left && operand_right == operator_right
                                    raise "Both #{operand.fixity} and #{operator.fixity} versions of #{name} yield the same outcome; this is an unresolvable ambiguity."
                                end
                            end
                            all_variants.delete(operand)
                            all_variants.delete(operator)
                        end
                    end
                    if all_variants.any?
                        raise "We will never pick #{all_variants.map { |v| v.fixity }.join(", ")} for term #{name}"
                    end
                end

                def validate!
                    raise "expression variant of #{name} must have no operands" if expression && !expression.expression?
                    raise "infix variant of #{name} must have left and right operands" if infix && !infix.infix?
                    raise "prefix variant of #{name} must have only right operand" if prefix && !prefix.prefix?
                    raise "postfix variant of #{name} must have only left operand" if postfix && !postfix.postfix?
                    raise "filler variant of #{name} must actually be filler, not #{filler.fixity}" if filler && !filler.filler?
                    raise "filler variant of #{name} will never be chosen" if filler && expression && infix && prefix && postfix

                    if (infix && infix.right.opens_indent_block?) || (prefix && prefix.right.opens_indent_block?)
                        raise "#{name} has an infix or prefix that opens an indent block, but also has a postfix or expression variant. This ambiguity is not supported by the parser." if postfix || expression
                        # If both infix and prefix are defined, and only one opens an indent block, we're still OK,
                        # because infix and prefix together are not ambiguous (we know which to pick without looking
                        # ahead to see what's on the right).
                    end
                end
            end
        end
    end
end
