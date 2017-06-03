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

            def space?
                (postfix? && postfix.space?) || (prefix? && prefix.space?)
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

            def resolve(state, term_start, term_end, next_is_space)
                space = leading_or_trailing_space(state, next_is_space)
                output.debug "- token #{name}, prefer #{state.prefer_operand_next? ? "operand" : "operator"}#{space ? ", #{space} space" : ""}"

                # Get term start/end, figure out if we have leading or trailing space around this term.
                state.syntax_tree.append_line(term_end) if name == :newline

                # Decide what we'll choose if the right side is operand, or if it's operator.
                action, prefer_operand_next, if_operand, if_operator =
                    next_state(state.prefer_operand_next?, space)

                output.debug "  action: #{action}, next: #{prefer_operand_next}, if_operand: #{if_operand.fixity}, if_operator: #{if_operator.fixity}"

                # Handle the left side according to what we've been told.
                case action
                when :resolve_left
                    resolve_left(state, if_operand.left_is_operand?)
                when :swap
                    state.swap_unresolved
                when nil
                else
                    raise "Unknown action #{action}"
                end

                # Append the actual term if unambiguous.
                if if_operand == if_operator
                    append_term(state, term_start, term_end, if_operand)
                else
                    if !if_operand.space?
                        state.if_operand_next ||= []
                        state.if_operand_next << [ term_start, term_end, if_operand ]
                    end
                    if !if_operator.space?
                        state.if_operator_next ||= []
                        state.if_operator_next << [ term_start, term_end, if_operator ]
                    end
                end

                state.prefer_operand_next = prefer_operand_next
                state.prev_is_space = space? unless term_start == term_end

                # Insert empty/apply (or ambiguous empty/apply) if there is a need
                grammar.border.resolve(state, term_end, term_end, next_is_space) if if_operand.right_is_operand? || !if_operator.right_is_operand?
            end

            private

            def append_term(state, term_start, term_end, type)
                if !type.space?
                    last_term = state.syntax_tree[-1]
                    output.debug "Appending #{type} (#{type.fixity})"
                    term = state.syntax_tree.append(term_start, term_end, type)
                    associate(term)
                end
            end

            def resolve_left(state, left_is_operand)
                resolved = state.resolve(left_is_operand)
                if resolved
                    output.debug "  left sides are both #{left_is_operand ? "operand" : "operator"}, resolving to #{resolved.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}"
                    resolved.each do |term_start, term_end, type|
                        append_term(state, term_start, term_end, type)
                    end
                end
            end

            def leading_or_trailing_space(state, next_is_space)
                return nil if space?
                return :leading  if state.prev_is_space && !next_is_space
                return :trailing if !state.prev_is_space && next_is_space
            end

            #
            # Associates an operator with the rest of the tree by setting its parent and
            # child correctly.
            #
            # @param [Term] The term in the tree to associate.
            #
            def associate(term)
                parent = term.previous_term
                return unless parent
                # If we have room for left children, pick the widest left child we can.
                type = term.type
                if type.left
                    while parent && type.left_accepts_operand?(parent.type)
                        left_operand, parent = parent, parent.parent
                    end
                    if !left_operand
                        raise internal_error(term, "#{term} (#{term.type.fixity} #{type}) cannot have left child #{parent} (#{parent.type.fixity} #{parent.type})!")
                    end
                    # If we are a close parentheses, and the chosen parent is our open parentheses (hopefully!),
                    # we make ourselves the parent of the open, and take the open's parent ourselves.
                    if type.left.opened_by
                        if parent && type.left.opened_by == parent.type
                            left_operand, parent = parent, parent.parent
                        else
                            raise unmatched_close(parent, term)
                        end
                    elsif parent && !parent.type.right_accepts_operand?(type)
                        raise internal_error(term, "#{term} cannot have parent #{parent}!")
                    end

                    left_operand.parent = term
                end
                term.parent = parent
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

                next_state_cache[index] = begin
                    space = nil if space?
                    if_operand, if_operator = [ false, true ].map do |right_operand|
                        # Handle compound terms: we prefer leading / trailing terms to be expressions, unless they cannot be.
                        left_operand = (space == :leading ? true : prefer_operand)
                        right_operand = true if space == :trailing
                        # no issue > issue on left > issue on right > both
                        variant( left_operand,  right_operand) ||
                        variant(!left_operand,  right_operand) ||
                        variant( left_operand, !right_operand) ||
                        variant(!left_operand, !right_operand)
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

            attr_reader :next_state_cache

            def cache_next_state!
                @next_state_cache = []
                all_variants = variants.to_set
                [ false, true ].each do |left|
                    [ nil, :leading, :trailing ].each do |space|
                        resolve, prefer_operand, operand, operator = next_state(left, space)
                        all_variants.delete(operand)
                        all_variants.delete(operator)
                    end
                end
                if all_variants.any?
                    raise "We will never pick #{all_variants.map { |v| v.fixity }.join(", ")} for term #{name}"
                end
            end

            def validate!
                variants.each do |variant|
                    raise "#{variant.fixity} variant of #{name} is not a TermType!" unless variant.is_a?(TermType)
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
