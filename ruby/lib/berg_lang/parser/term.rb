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

            def inspect
                "#{string.inspect} (#{type ? type.name : nil})@#{source_range})"
            end

            def to_s
                string
            end

            def ==(other)
                syntax_tree == other.syntax_tree && index == other.index
            end

            def insert(token_start, token_end, type=nil)
                syntax_tree.insert(index, token_start, token_end, type)
            end

            def append(token_start, token_end, type=nil)
                syntax_tree.insert(index+1, token_start, token_end, type)
            end

            def parent_index
                syntax_tree.terms[index][3]
            end
            def parent
                syntax_tree[parent_index] if parent_index
            end
            def parent=(value)
                syntax_tree.terms[index][3] = value ? value.index : value
            end

            def type
                syntax_tree.terms[index][2]
            end
            def type=(value)
                syntax_tree.terms[index][2] = value
            end

            def previous_term
                syntax_tree[index-1] if index > 0
            end
            def next_term
                syntax_tree[index+1]
            end

            def left_operand
                term = previous_term
                term = term.parent while term && term.parent_index != index
                term
            end

            def right_operand
                term = next_term
                term = term.parent while term && term.parent_index != index
                term
            end

            def source_range
                SourceRange.new(syntax_tree, start, self.end)
            end

            def string
                source_range.string
            end

            def start
                type.left && left_operand ? left_operand.start : syntax_tree.terms[index][0]
            end
            def end
                type.right && right_operand ? right_operand.end : syntax_tree.terms[index][1]
            end

            def expression_to_s
                if type.infix?
                    left = left_operand
                    right = right_operand
                    "(#{left ? left.expression_to_s : "missing"} #{type} #{right ? right.expression_to_s : "missing"})"
                elsif type.postfix?
                    left = left_operand
                    "#{left ? left.expression_to_s : "missing"}#{type.name == :eof ? "" : type}"
                elsif type.prefix?
                    right = right_operand
                    "#{type.name == :sof ? "" : type}#{right ? right.expression_to_s : "missing"}"
                else
                    string
                end
            end
        end
    end
end