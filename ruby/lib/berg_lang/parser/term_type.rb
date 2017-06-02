require_relative "term_type/side"
require_relative "term_type/variant"

module BergLang
    class Parser
        class TermType
            attr_reader :name
            attr_reader :string
            attr_reader :expression
            attr_reader :infix
            attr_reader :prefix
            attr_reader :postfix

            def initialize(name, string: nil, expression: nil, infix: nil, prefix: nil, postfix: nil)
                @name = name
                @string = string
                @expression = expression
                @infix = infix
                @prefix = prefix
                @postfix = postfix
                validate!
                cache_next_state!
            end

            def self.define(name, string: nil, left: nil, right: nil, space: nil)
                left = TermType::Side.new(**left) if left
                right = TermType::Side.new(**right) if right
                variant = TermType::Variant.new(name, string: string, left: left, right: right, space: space)
                if left
                    if right
                        self.new(name, string: string, infix: variant)
                    else
                        self.new(name, string: string, postfix: variant)
                    end
                else
                    if right
                        self.new(name, string: string, prefix: variant)
                    else
                        self.new(name, string: string, expression: variant)
                    end
                end
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

            def variant(left, right)
                if left
                    right ? expression : prefix
                else
                    right ? postfix : infix
                end
            end

            def +(term_type)
                raise "#{self} and #{term_type} are both expressions and cannot be combined" if term_type.expression? && expression?
                raise "#{self} and #{term_type} are both infix and cannot be combined" if term_type.infix? && infix?
                raise "#{self} and #{term_type} are both prefix and cannot be combined" if term_type.prefix? && prefix?
                raise "#{self} and #{term_type} are both postfix and cannot be combined" if term_type.postfix? && postfix?
                raise "Cannot combine differently named variants #{name} and #{term_type.name}!" unless name == term_type.name
                raise "Cannot share variants of #{name} with different strings (#{string.inspect} and #{term_typestring.inspect}!" unless string == term_type.string
                TermType.new(name, string: string, infix: term_type.infix || infix, expression: term_type.expression || expression, prefix: term_type.prefix || prefix, postfix: term_type.postfix || postfix)
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
            # over the variant that only fails once, over the variant that fails twice.
            #
            # @return [Boolean,Boolean] The preferred variant for operand and for operator
            # @example if_operand, if_operator = type.next_state(true, true)
            #
            def next_state(prefer_operand)
                next_state_cache[prefer_operand] ||= begin
                    if_operand, if_operator = [ false, true ].map do |right_operand|
                        # no issue > issue on left > issue on right > both
                        variant( prefer_operand,  right_operand) ||
                        variant(!prefer_operand,  right_operand) ||
                        variant( prefer_operand, !right_operand) ||
                        variant(!prefer_operand, !right_operand)
                    end

                    # If the right sides are the same (infix / prefix / expression /
                    # postfix / infix|prefix / expression|postfix), we can't influence
                    # the next outcome, so we just pick the one that is preferred.
                    if if_operand.right_is_operand? == if_operator.right_is_operand?
                        if_operator = if_operand = prefer_operand ? if_operand : if_operator
                        prefer_operand_next = !if_operand.right_is_operand?
                        action = :resolve_left

                    # If the left sides are the same (infix|postfix / expression|prefix),
                    # our preference can't be influenced by the prior preference, so
                    # prefer the one with the higher priority. 
                    elsif if_operand.left_is_operand? == if_operator.left_is_operand?
                        prefer_operand_next = if_operand.priority > if_operator.priority
                        action = :resolve_left

                    # If left and right are different (infix|expression / postfix|prefix),
                    # keep or swap prefer_operand.
                    elsif if_operand.left_is_operand? == if_operand.right_is_operand?
                        prefer_operand_next = !prefer_operand
                        action = :swap

                    else
                        prefer_operand_next = prefer_operand
                    end

                    [ action, prefer_operand_next, if_operand, if_operator ]
                end
            end

            private

            attr_reader :next_state_cache

            def cache_next_state!
                @next_state_cache = {}
                all_variants = variants.to_set
                [ false, true ].each do |left|
                    resolve, prefer_operand, operand, operator = next_state(left)
                    all_variants.delete(operand)
                    all_variants.delete(operator)
                end
                if all_variants.any?
                    raise "We will never pick #{all_variants.map { |v| v.fixity }.join(", ")} for term #{name}"
                end
            end

            def validate!
                variants.each do |variant|
                    raise "#{variant.fixity} variant of #{name} is not a Variant!" unless variant.is_a?(TermType::Variant)
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
