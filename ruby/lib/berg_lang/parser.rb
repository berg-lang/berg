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
                output.debug "- token #{type}, prefer #{state.prefer_operand_next? ? "operand" : "operator"}"
                state.syntax_tree.append_line(term_end, 0) if type == terms.newline

                resolve(state, term_start, term_end, type)
            end

            state.syntax_tree
        end

        private

        include SyntaxErrors

        def resolve(state, term_start, term_end, type)
            # Decide what we'll choose if the right side is operand, or if it's operator.
            action, prefer_operand_next, if_operand, if_operator =
                type.next_state(state.prefer_operand_next?)

            output.debug "  - action: #{action}, next: #{prefer_operand_next}, if_operand: #{if_operand.fixity}, if_operator: #{if_operator.fixity}"
            # Handle the left side according to what we've been told.
            case action
            when :resolve_left
                resolve_left(state, if_operand.left_is_operand?)
            when :swap
                state.swap_unresolved
            end

            # Append the actual term if unambiguous.
            if if_operand == if_operator
                append_term(state, term_start, term_end, if_operand)
            else
                state.if_operand_next << [ term_start, term_end, if_operand ] unless if_operand.space?
                state.if_operator_next << [ term_start, term_end, if_operator ] unless if_operator.space?
            end
            state.prefer_operand_next = prefer_operand_next

            # Insert empty/apply (or ambiguous empty/apply) if there is a need
            resolve(state, term_end, term_end, terms.border) if if_operand.right_is_operand? || !if_operator.right_is_operand?
        end

        def resolve_left(state, left_is_operand)
            resolved = state.resolve(left_is_operand)
            if resolved.any?
                output.debug "  left sides are both #{left_is_operand ? "operand" : "operator"}, resolving to #{resolved.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}"
                resolved.each do |term_start, term_end, type|
                    append_term(state, term_start, term_end, type)
                end
            end
        end

        def append_term(state, term_start, term_end, type)
            last_term = state.syntax_tree[-1]
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
                        raise mismatched_open_close(parent, term)
                    end
                elsif parent && !parent.type.right_accepts_operand?(type)
                    raise internal_error(term, "#{term} cannot have parent #{parent}!")
                end

                left_operand.parent = term
            end
            term.parent = parent
        end
    end
end
