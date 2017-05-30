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
            state = State.new(source, prefer_operand_next: true, insert_if_non_preferred: terms.apply)

            # Read each token from the input
            term_end = scanner.index
            state.syntax_tree.append_line(term_end, 0)
            while type = scanner.next
                term_start, term_end = term_end, scanner.index
                puts "- token #{type}"
                state.syntax_tree.append_line(term_end, 0) if type == terms.newline

                resolve(state, term_start, term_end, type)
            end

            state.syntax_tree
        end

        private

        include SyntaxErrors

        def resolve(state, term_start, term_end, type)
            # Decide what we'll choose if the right side is operand, or if it's operator.
            if_operand, if_operator = type.preferred_variants(
                state.prefer_operand_next?,
                state.insert_if_non_preferred == terms.empty
            )

            # If left sides are identical, we can resolve everything to the left.
            if_operand_left  =  if_operand.filler?  || if_operand.left_is_operand?
            if_operator_left = !if_operator.filler? && if_operator.left_is_operand?
            if if_operand_left == if_operator_left
                insert_type, resolved = state.resolve(if_operand_left)
                output.debug "  left sides are both #{if_operand_left ? "operand" : "operator"}, resolving" if insert_type || resolved.any?
                if insert_type
                    next_term_start = terms.any? ? terms.first[0] : term_start
                    append_term(state, next_term_start, next_term_start, insert_type)
                end
                resolved.each do |term_start, term_end, type|
                    append_term(state, term_start, term_end, type)
                end

                # If both sides are identical, append. Otherwise, it's unresolved.
                if if_operand == if_operator
                    puts "  new state: #{type.right_is_operand? ? "operator" : "operand"}"
                    append_term(state, term_start, term_end, if_operand)
                    state.set(!type.right_is_operand?, type.right_is_operand? ? terms.empty : terms.apply)
                else
                    state.set_unresolved(term_start, term_end, if_operand, if_operator)
                end
            elsif if_operand.filler? && if_operator.filler?
            else
                state.append_unresolved(term_start, term_end, if_operand, if_operator)
            end
        end

        def append_term(state, term_start, term_end, type)
            output.debug "Appending #{type} (#{type.fixity})"
            term = state.syntax_tree.append(term_start, term_end, type)
            associate(term)
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
                    raise internal_error(term, "#{term} (#{term.type.fixity}) cannot have left child #{parent} (#{parent.type.fixity})!")
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
