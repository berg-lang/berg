require_relative "line_data"
require_relative "syntax_tree"

module BergLang
    class Parser
        class ParseResult
            attr_reader :source
            attr_reader :line_data
            attr_reader :syntax_tree

            def initialize(source, output)
                @source = source
                @output = output
                @line_data = LineData.new(source)
                @syntax_tree = SyntaxTree.new(source, line_data)
            end

            def root
                syntax_tree.root
            end

            #
            # Appends a single term to the output.
            #
            def append_term(token_start, token_end, statement_indent, type)
                if type.significant?
                    output.debug "Appending #{type} (#{type.fixity})"
                    term = syntax_tree.append(token_start, token_end, statement_indent, type)
                    associate(term)
                end
            end

            private

            attr_reader :output

            #
            # Associates an operator with the rest of the tree by setting its parent and
            # child correctly.
            #
            # @param [Term] The term in the tree to associate.
            #
            def associate(term)
                parent = term.previous_term
                return unless parent

                # If the previous term has an indented variant, check if its immediate
                # child is indented or not.
                if parent.type.indented_variant
                    if parent.statement_indent < term.statement_indent
                        output.debug "Picking indented variant of #{parent}"
                        parent.type = parent.type.indented_variant
                    else
                        output.debug "Not picking indented variant of #{parent}: #{parent.statement_indent.inspect} (#{parent.type}) < #{term.statement_indent.inspect} (#{term.type})"
                    end
                end

                # If we have room for left children, pick the widest left child we can.
                if term.type.left
                    while parent && term.left_accepts_operand?(parent)
                        left_operand, parent = parent, parent.parent
                    end
                    if !left_operand
                        raise internal_error(term, "#{term} (#{term.type.fixity} #{type}) cannot have left child #{parent} (#{parent.type.fixity} #{parent.type})!")
                    end
                    # If we are a close parentheses, and the chosen parent is our open parentheses (hopefully!),
                    # we make ourselves the parent of the open, and take the open's parent ourselves.
                    if term.type.left.opened_by
                        if parent && term.type.left.opened_by == parent.type
                            left_operand, parent = parent, parent.parent
                        else
                            puts "Unmatched close #{parent}, #{term}"
                            raise unmatched_close(parent, term)
                        end
                    elsif parent && !parent.right_accepts_operand?(term)
                        raise internal_error(term, "#{term} cannot have parent #{parent}!")
                    end

                    left_operand.parent = term
                end
                term.parent = parent
            end
        end
    end
end
