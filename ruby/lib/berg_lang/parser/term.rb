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
                syntax_tree.terms[index][PARENT_INDEX]
            end
            def parent
                syntax_tree[parent_index] if parent_index
            end
            def parent=(value)
                syntax_tree.terms[index][PARENT_INDEX] = value ? value.index : value
            end

            def type
                syntax_tree.terms[index][TERM_TYPE]
            end
            def type=(value)
                syntax_tree.terms[index][TERM_TYPE] = value
            end

            def previous_term
                syntax_tree[index-1] if index > 0
            end
            def next_term
                syntax_tree[index+1]
            end

            def left_accepts_operand?(term)
                return false if term.type.statement_boundary == :nest && term.statement_indent <= statement_indent && !type.left.opened_by
                type.left_accepts_operand_type?(term.type)
            end

            def right_accepts_operand?(term)
                return true if type.statement_boundary == :nest && statement_indent < term.statement_indent
                type.right_accepts_operand_type?(term.type)
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
                SourceRange.new(syntax_tree.line_data, start, self.end)
            end

            def string
                source_range.string
            end

            def statement_indent
                syntax_tree.terms[index][STATEMENT_INDENT]
            end

            def start
                type.left && left_operand ? left_operand.start : syntax_tree.terms[index][TOKEN_START]
            end
            def end
                type.right && right_operand ? right_operand.end : syntax_tree.terms[index][TOKEN_END]
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

            private

            TOKEN_START = 0
            TOKEN_END = 1
            STATEMENT_INDENT = 2
            TERM_TYPE = 3
            PARENT_INDEX = 4
        end
    end
end