require_relative "output"
require_relative "parser/berg_terms"
require_relative "parser/state"
require_relative "parser/syntax_errors"
require_relative "parser/syntax_tree"
require_relative "parser/scanner"

module BergLang
    #
    # Parses Berg.
    #
    # Disambiguation rules:
    # 1. The earliest thing we can insert, wins.
    # 2. infix>postfix and expression>prefix if either one works fine.
    class Parser
        attr_reader :source
        attr_reader :syntax_tree
        attr_reader :output

        def initialize(source, output: Output.new(STDOUT))
            @source = source
            @output = output
        end

        def terms
            BergTerms
        end

        #
        # Parses all terms from the source.
        #
        def parse
            scanner = Scanner.new(self)
            state = State.new(source, prefer_next_to_be_operand: true, insert_if_need_operator: terms.apply, insert_if_need_operand: terms.empty)

            term_start = scanner.index
            while type = scanner.next
                term_end = scanner.index
                resolve(state, term_start, term_end, type)
            end
        end

        private

        def resolve(state, term_start, term_end, type)
            if_next_is_operand, if_next_is_operator = choose_types(state, type)

            if if_next_is_operand
                if if_next_is_operator
                    resolve_ambiguous(state, term_start, term_end, if_next_is_operand, if_next_is_operator)
                else
                    resolve_unambiguous(state, term_start, term_end, if_next_is_operand)
                end
            else
                if if_next_is_operator
                    resolve_unambiguous(state, term_start, term_end, if_next_is_operator)
                else
                    raise syntax_errors.internal_error(term_start, term_end, type, "No operator, operand or filler for #{type}")
                end
            end
        end

        def choose_types(state, type)
            # When we have both right sides match, we know there is no insert, so we can safely pick whichever fits the left best. TODO not true. filler!
            case state.prefer_next_to_be_operand?
            when true
                if_next_is_operand  = type.prefix     || type.filler  || type.infix
                if_next_is_operator = type.expression || type.filler  || type.postfix
            when false
                if_next_is_operand  = type.infix      || type.filler  || type.prefix
                if_next_is_operator = type.postfix    || type.filler  || type.expression
            when nil
                # When we can't use the ambiguity of the situation to guide us, we invoke the infix>prefix
                # and expression>postfix rule.
                if_next_is_operand  = type.infix      || type.prefix  || type.filler
                if_next_is_operator = type.expression || type.postfix || type.filler
            end
            [ if_next_is_operand, if_next_is_operator ]
        end

        def resolve_unambiguous(state, term_start, term_end, type)
            state.resolved_terms(term_start, type).each { |term| append(*term) }
            state.reset(prefer_next_to_be_operand: !type.right_is_operand?)
            append(term_start, term_end, type)
        end

        def resolve_ambiguous(state, term_start, term_end, if_next_is_operand, if_next_is_operator)
            # If both sides are filler, it's actually unambiguous. Return.
            return if if_next_is_operand.filler? && if_next_is_operator.filler?

            # If the left hand side is unambiguous even though the right side is not (infix|postfix,
            # prefix|expression, filler|expression, infix|filler), resolve the left side and reset if_operand/if_operator.
            if_next_is_operand_is_operand = if_next_is_operand.filler? ? true : if_next_is_operand.left_is_operand?
            if_next_is_operator_is_operand = if_next_is_operator.filler? ? false : if_next_is_operator.left_is_operand?
            if if_next_is_operand_is_operand == if_next_is_operator_is_operand
                state.resolved_terms(term_start, type).each { |term| append(*term) }
                state.reset(prefer_next_to_be_operand: nil)
            
            # If we have infix|expression, flip if_operand/if_operator since it flips what we want next.
            elsif if_next_is_operand.infix? && if_next_is_operator.expression?
                state.swap

            # If we have prefix|postfix, filler|postfix, or prefix|filler we don't mess with anything, no resolutions
            # can be done and nothing needs to be swapped.
            end

            state.append(term_start, term_end, if_next_is_operand, if_next_is_operator)
        end

        def append(to_append)
            to_append.each do |term_start, term_end, type|
                next if term.filler?
                term = syntax_tree.append(term_start, term_end, type)
                associate(term)
            end
        end

        #
        # Associates an operator with the rest of the tree by setting its parent and child correctly.
        #
        # @param [Term] The term in the tree to associate.
        #
        def associate(term)
            parent = term.previous_term
            return unless parent
            type = term.type
            # If we have room for left children, pick the widest left child we can.
            if type.left
                while parent && type.left.accepts_operand?(parent.type)
                    left_operand, parent = parent, parent.parent
                end
                if !left_operand
                    raise internal_error(term, "#{term.inspect} cannot have left child #{parent.inspect}!")
                end
                # If we are a close parentheses, and the chosen parent is our open parentheses (hopefully!),
                # we make ourselves the parent of the open, and take the open's parent ourselves.
                if type.left.opened_by
                    if type.left.opened_by == parent.type
                        left_operand, parent = parent, parent.parent
                    else
                        raise mismatched_open_close(parent, term)
                    end
                elsif parent && !parent.type.right.accepts_operand?(type)
                    raise internal_error(term, "#{term} cannot have parent #{parent}!")
                end

                left_operand.parent = term
            end
            term.parent = parent
        end
    end
end
