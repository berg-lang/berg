module BergLang
    class Parser
        #
        # A single term (operator or expression) in the syntax tree.
        #
        class Term
            attr_reader :syntax_tree
            attr_reader :index

            def initialize(syntax_tree, index)
                @syntax_tree = syntax_tree
                @index = index
            end

            def parent
                syntax_tree[index][3]
            end
            def parent=(value)
                syntax_tree[index][3] = value
            end

            def type
                syntax_tree.nodes[index][2]
            end
            def type=(value)
                syntax_tree.nodes[index][2] = value
            end

            def previous_term
                syntax_tree[index-1] if index > 0
            end
            def next_term
                syntax_tree[index+1]
            end

            def left_operand
                term = previous_term
                term = term.parent while term.parent != self
                term
            end
            def right_operand
                term = next_term
                term = term.parent while term.parent != self
                term
            end

            def source_range
                SourceRange.new(syntax_tree.source, start, self.end)
            end

            def token_range
                token_start, token_end = syntax_tree.nodes[index]
                SourceRange.new(syntax_tree.source, token_start, token_end)
            end

            def start
                type.left.is_operator? ? left_operand.start : syntax_tree.nodes[index][0]
            end
            def end
                type.right.is_operator? ? right_operand.end : syntax_tree.nodes[index][1]
            end
        end
    end
end