require_relative "term_type"

module BergLang
    class Parser
        class TokenType
            attr_reader :grammar
            attr_reader :name
            attr_reader :string
            attr_reader :expression
            attr_reader :infix
            attr_reader :prefix
            attr_reader :postfix

            def initialize(grammar, name, string: nil, expression: nil, infix: nil, prefix: nil, postfix: nil)
                @grammar = grammar
                @name = name
                @string = string
                @expression = expression
                @infix = infix
                @prefix = prefix
                @postfix = postfix
                validate!
                cache_next_state!
            end

            def output
                grammar.output
            end

            def to_s
                name.is_a?(String) ? name : name.inspect
            end

            def infix?
                !!infix
            end
            def expression?
                !!expression
            end
            def prefix?
                !!prefix
            end
            def postfix?
                !!postfix
            end

            def space
                variants.first.space
            end

            def space?
                variants.first.space?
            end

            def significant?
                variants.any? { |variant| variant.significant? }
            end

            def variant(left, right)
                if left
                    right ? expression : prefix
                else
                    right ? postfix : infix
                end
            end

            def variants
                return enum_for(:variants) if !block_given?
                yield expression if expression?
                yield infix if infix?
                yield prefix if prefix?
                yield postfix if postfix?
            end

            #
            # Determine the best variant when the next operand is an operator, and when
            # it is an operand.
            #
            # Pick the variant that fits perfectly, over the variant that fails earliest,
            # over the variant that fails twice.
            #
            # @return [Boolean,Boolean] The preferred variant for operand and for operator
            # @example if_operand, if_operator = type.next_state(true, true)
            #
            def next_state(prefer_operand, space)
                raise "prefer_operand cannot be #{prefer_operand.class}" unless prefer_operand == true || prefer_operand == false
                case space
                when :leading
                    index = 0x2 + (prefer_operand ? 0x0 : 0x1)
                when :trailing
                    index = 0x4 + (prefer_operand ? 0x0 : 0x1)
                when nil
                    index = 0x6 + (prefer_operand ? 0x0 : 0x1)
                else
                    raise "yarr space cannot be #{space}"
                end

                next_state_cache[index] ||= begin
                    space = nil if space?
                    if_operand = preferred_variant(space, prefer_operand, right_operand: false)
                    if_operator = preferred_variant(space, prefer_operand, right_operand: true)

                    # If the right sides are the same (infix / prefix / expression /
                    # postfix / infix|prefix / expression|postfix), we can't influence
                    # the next outcome, so we just pick the one that is preferred.
                    if if_operand.right_is_operand? == if_operator.right_is_operand?
                        if prefer_operand
                            if_operator = if_operand
                        else
                            if_operand = if_operator
                        end
                        prefer_operand_next = !if_operand.right_is_operand?
                        action = :resolve

                    # If the left sides are the same (infix|postfix / expression|prefix),
                    # our preference can't be influenced by the prior preference, so
                    # prefer the one with the higher priority. 
                    elsif if_operand.left_is_operand? == if_operator.left_is_operand?
                        prefer_operand_next = if_operand.priority > if_operator.priority
                        action = :resolve

                    # Left and right are different (infix|expression / postfix|prefix).
                    # If they are infix/expression, swap prefer_operand.
                    elsif if_operand.left_is_operand? == if_operand.right_is_operand?
                        prefer_operand_next = !prefer_operand
                        action = :swap

                    # If it's postfix/prefix, keep prefer_operand.
                    else
                        prefer_operand_next = prefer_operand
                    end

                    [ action, prefer_operand_next, if_operand, if_operator ]
                end
            end

            private

            attr_reader :next_state_cache

            def preferred_variant(space, prefer_operand, right_operand:)
                # Handle compound terms: we prefer leading / trailing terms to be expressions, unless they cannot be.
                left_operand = (space == :leading ? true : prefer_operand)
                right_operand = true if space == :trailing
                # no issue > issue on left > issue on right > both
                variant( left_operand,  right_operand) ||
                variant(!left_operand,  right_operand) ||
                variant( left_operand, !right_operand) ||
                variant(!left_operand, !right_operand)
            end

            def cache_next_state!
                @next_state_cache = []
                all_variants = variants.to_set
                [ false, true ].each do |left|
                    [ nil, :leading, :trailing ].each do |space|
                        resolve, prefer_operand, if_operand, if_operator = next_state(left, space)
                        all_variants.delete(if_operand)
                        all_variants.delete(if_operator)
                    end
                end
                if all_variants.any?
                    raise "We will never pick #{all_variants.map { |v| v.fixity }.join(", ")} for term #{name}"
                end
            end

            def validate!
                space = variants.first.space
                variants.each do |variant|
                    raise "#{variant.fixity} variant of #{name} is not a TermType!" unless variant.is_a?(TermType)
                    raise "All variants of #{name} must have the same value for `space`! #{variant.fixity} has #{variant.space.inspect}, while #{variants.first.fixity} has #{space.inspect}" unless variant.space == space
                end
                raise "Must have non-nil, non-empty name: #{name.inspect}" if name.nil? || name == ""
                raise "must have at least one variant of #{name}!" if !variants.any?
                raise "expression variant of #{name} must have no operands" if expression && !expression.expression?
                raise "infix variant of #{name} must have left and right operands" if infix && !infix.infix?
                raise "prefix variant of #{name} must have only right operand" if prefix && !prefix.prefix?
                raise "postfix variant of #{name} must have only left operand" if postfix && !postfix.postfix?

                if infix? && postfix? && infix.priority == postfix.priority
                    raise "infix and postfix variants of #{name} have the same priority (#{infix.priority}! Set them to different priorities, or we won't be able to disambiguate in some cases."
                end
                if expression? && prefix? && expression.priority == prefix.priority
                    raise "expression and prefix variants of #{name} have the same priority (#{expression.priority}! Set them to different priorities, or we won't be able to disambiguate in some cases."
                end
            end
        end
    end
end
