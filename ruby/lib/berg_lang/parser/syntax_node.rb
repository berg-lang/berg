module BergLang
    module Parser
        class SyntaxNode < Struct[:syntax_tree, :syntax_tree_index]
            # Index to the type property in the syntax tree
            TYPE = 0
            INDEX = 1
            STRING = 2
            PARENT = 3

            def token_data
                syntax_tree[syntax_tree_index]
            end
            def token_type
                token_data[TYPE]
            end
            def token_string
                token_data[STRING]
            end
            def token_start
                token_data[INDEX]
            end
            def token_end
                token_start+token_string.size
            end
            def token_range
                token_start...token_end
            end
            def token_parent_index
                token_data[PARENT]
            end

            def parent_operator
                return nil unless token_parent_index
                syntax_tree[token_parent_index]
            end
            def next_token
                syntax_tree[syntax_tree_index+1]
            end
            def previous_token
                syntax_tree[syntax_tree_index-1] if syntax_tree_index > 0
            end
            def left_operand
                if token_type.left_is_operator?
                    candidate = syntax_tree[syntax_tree_index - 1]
                    while candidate.token_parent_index != index
                        candidate = candidate.parent 
                    end
                    candidate
                end
            end
            def right_operand
                if token_type.right_is_operator?
                    candidate = syntax_tree[syntax_tree_index + 1]
                    while candidate.token_parent_index != index
                        candidate = candidate.parent
                    end
                    candidate
                end
            end
            def leftmost_term
                # If our we are a right child, our leftmost term is always the term to the right of our parent.
                # If our we are a left child, our leftmost term is the leftmost term of our parent.
                # If our parent is the root, our leftmost term is the very first token.
                if token_type.left_is_operator?
                    child = self
                    # Find the root or the first parent operator that we are a *right* child of.
                    while child.left_side_of_parent_operator?
                        child = child.parent_operator
                    end
                    parent = child.parent_operator
                    return syntax_tree_node.first_token if !parent
                    parent.next_token
                end
            end
            def rightmost_term
                # If we are a left child, our rightmost term is always the term to the left to our parent.
                # If we are a right child, our rightmost term is the rightmost term of our parent.
                # If our parent is the root, our rightmost term is the very last token.
                if token_type.left_is_operator?
                    child = self
                    # Find the root or the first parent operator that we are a *left* child of.
                    while child.right_side_of_parent_operator?
                        child = child.parent_operator
                    end
                    parent = child.parent_operator
                    return syntax_tree_node.last_token if !parent
                    parent.previous_token
                end
            end
            def left_side_of_parent_operator?
                token_parent_index && syntax_tree_index < token_parent_index
            end
            def right_side_of_parent_operator?
                token_parent_index && token_parent_index < syntax_tree_index
            end
            def start
                leftmost_term.token_start
            end
            def end
                rightmost_term.token_end
            end
            def range
                start...end
            end
        end
    end
end
