module BergLang
    module Parser
        #
        # Represents an operation in a syntax tree.
        #
        # Either an expression, or an operator.
        #
        class Operation < Struct[:syntax_tree, :syntax_tree_index]
            def initialize(syntax_tree, syntax_tree_index)
                @syntax_tree = syntax_tree
                @syntax_tree_index = syntax_tree_index
            end

            #
            # The syntax tree this operation is in
            #
            attr_reader :syntax_tree

            #
            # The index of this operation in the syntax tree
            #
            attr_reader :syntax_tree_index

            #
            # The type of this token.
            #
            # @return [TokenType] The type of this token.
            #
            def type
                token.type
            end

            #
            # The token representing this operation.
            #
            # @return [Token] The token representing this operation.
            #
            def token
                @token ||= syntax_tree[syntax_tree_index]
            end

            #
            # Next operation in lexical order
            #
            # @return [Operation] The next operation, or `nil` if no more operations.
            #
            def next
                syntax_tree[syntax_tree_index+1]
            end

            #
            # Previous operation in lexical order
            #
            # @return [Operation] the previous operation, or `nil` if this is the first operation.
            def previous
                syntax_tree[syntax_tree_index-1] if syntax_tree_index > 0
            end

            def parent_operator
                return nil unless token.parent_index
                syntax_tree[token.parent_index]
            end
            def left_operand
                return nil unless [:infix,:postfix].include?(token.type.operation_type)

                candidate = syntax_tree[syntax_tree_index - 1]
                while candidate.token.parent_index != index
                    candidate = candidate.parent 
                end
                candidate
            end
            def right_operand
                return nil unless [:infix,:prefix].include?(token.type.operation_type)

                candidate = syntax_tree[syntax_tree_index + 1]
                while candidate.token.parent_index != index
                    candidate = candidate.parent
                end
                candidate
            end
            def leftmost_term
                # If our we are a right child, our leftmost term is always the term to the right of our parent.
                # If our we are a left child, our leftmost term is the leftmost term of our parent.
                # If our parent is the root, our leftmost term is the very first token.
                return nil unless [:infix,:postfix].include?(token.type.operation_type)

                child = self
                # Find the root or the first parent operator that we are a *right* child of.
                while child.left_side_of_parent_operator?
                    child = child.parent_operator
                end
                parent = child.parent_operator
                return syntax_tree_operation.first_token if !parent
                parent.next_token
            end
            def rightmost_term
                # If we are a left child, our rightmost term is always the term to the left to our parent.
                # If we are a right child, our rightmost term is the rightmost term of our parent.
                # If our parent is the root, our rightmost term is the very last token.
                return nil unless [:infix,:prefix].include?(token.type.operation_type)

                child = self
                # Find the root or the first parent operator that we are a *left* child of.
                while child.right_side_of_parent_operator?
                    child = child.parent_operator
                end
                parent = child.parent_operator
                return syntax_tree_operation.last_token if !parent
                parent.previous_token
            end
            def left_side_of_parent_operator?
                token.parent_index && syntax_tree_index < token.parent_index
            end
            def right_side_of_parent_operator?
                token.parent_index && token.parent_index < syntax_tree_index
            end
            def start
                leftmost_term.token.range.start
            end
            def end
                rightmost_term.token.range.end
            end
            def range
                start...end
            end
        end
    end
end
