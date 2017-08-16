module BergLang
    module Parser
        class SyntaxTreeBuilder
            attr_reader :output
            attr_reader :syntax_tree
            attr_reader :available_operand
            attr_reader :open_statements
            attr_reader :margin

            def initialize(source_data, output)
                @output = output
                @syntax_tree = SyntaxTree.new(source_data)
                @available_operand = nil
                @open_statements = []
                @margin = nil
            end

            def source_data
                syntax_tree.source_data
            end

            def need_operator?
                !!available_operand
            end

            def mark_visible_indent(visible_indent)
                # Close any open blocks whose indent is <= the block's parent indent
                to_close = nil
                open_indented_blocks.each do |parent_operator, parent_indent|
                    if visible_indent <= parent_indent
                        to_close = parent_operator
                        @margin = parent_indent
                        break
                    end
                end

                close_operator(to_close) if to_close
            end

            #
            # Append the given token into the AST.
            #
            # Uses precedence to put it in the right place in the tree.
            #
            def append(token_type, token_index, string)
                if token_type.opens_indented_block?
                    if indent
                if indent
                    if margin.nil?
                        @margin = indent
                    elsif indent == margin
                if need_operator? != token_type.left_is_operator?
                    
                end

                # Find the widest thing we can have as a child
                if token_type.left_is_operator?
                    if syntax_tree.last_token && !syntax_tree.last_token.right_is_operator?
                        raise "INTERNAL ERROR: added operator #{token_type} (#{string.inspect}@#{token_index}) when last token is #{syntax_tree.last_token}"
                    end

                    left_operand = available_operand
                    if can_have_left_child?(token_type, left_operand, indent)
                        while can_have_left_child?(token_type, left_operand.parent, indent)
                            left_operand = left_operand.parent
                        end

                        node = syntax_tree.append(token_type, token_index, string, left_operand.token_parent_index)
                        left_operand.parent = node
                    else
                        node = syntax_tree.append(token_type, )
                    end
                else
                    if syntax_tree.last_token && syntax_tree.last_token.right_is_operator?
                        raise "INTERNAL ERROR: added non-operator #{token_type} (#{string.inspect}@#{token_index}) when last token is #{syntax_tree.last_token}"
                    end
                end


                node = syntax_tree.append(token_type, token_start, token_string, indent)

                    parent = available_operand || syntax_tree.last_token
                # If there is an open_operator,  open_operator


                # If
                if token_type.left_is_operator?

            end

            private

            def close_operator(operator)
                while open_indented_blocks.any? && open_indented_blocks[-1][0].syntax_tree_index >= operator.syntax_tree_index
                    open_indented_blocks.pop
                end
                @available_operand = operator
            end

            def desired_parent(token_type, indent)
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
                        raise internal_error(term, "#{term} (#{term.type.operation_type} #{type}) cannot have left child #{parent} (#{parent.type.operation_type} #{parent.type})!")
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
