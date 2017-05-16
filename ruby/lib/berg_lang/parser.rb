require_relative "output"
require_relative "parser/berg_terms"
require_relative "parser/syntax_errors"
require_relative "parser/syntax_tree"
require_relative "parser/resolver"

module BergLang
    #
    # Parses Berg.
    #
    class Parser
        attr_reader :source
        attr_reader :output
        attr_reader :syntax_tree

        def initialize(source, output: Output.new(STDOUT))
            @source = source
            @output = output
            @syntax_tree = SyntaxTree.new(source)
        end

        def terms
            BergTerms
        end

        #
        # Parses, resolves, and creates a syntax tree for a source.
        #
        def parse
            resolver = Resolver.new(self)
            left_accepts_operand = true
            expect_indent_block = false
            while term = resolver.parse_next(left_accepts_operand, expect_indent_block, Resolver::NONE)
                associate_operators(term)

                if right = syntax_tree[-1].type.right
                    left_accepts_operand = true
                    expect_indent_block = right.opens_indent_block?
                else
                    left_accepts_operand = false
                end
            end
        end

        private

        include SyntaxErrors

        #
        # Associates an operator (and any operators after it) with the rest of the tree by setting its
        # parent and child correctly.
        #
        # @param [Term] The term in the tree to associate.
        #
        def associate_operators(term)
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

            # If there is another term to associate, take care of it.
            next_term = term.next_term
            associate_operators(next_term) if next_term
        end
    end
end
