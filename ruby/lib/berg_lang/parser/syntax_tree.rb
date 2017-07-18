require_relative "term"

module BergLang
    class Parser
        #
        # Represents the entire syntax tree of a given Source.
        #
        # The set of terms is an array with parent pointers to indicate expression children.
        # Terms are in the same order as the source, and are complete--anything between one term's
        # end and the next term's start is insignificant whitespace.
        #
        # Tree traversal is done by looking at the parent index of each term. If the parent is to the
        # left, it is a right operand of its parent; if it is to the right, it is a left operand.
        #
        # Because the terms are stored in lexical order, one's left operand is always one of the parents
        # of the previous term; and one's right operand is always one of the parents of the next term.
        #
        class SyntaxTree
            attr_reader :source
            attr_reader :line_data
            attr_reader :terms

            def initialize(source, line_data)
                @source = source
                @line_data = line_data
                @terms = []
            end

            include Enumerable

            def to_s
                terms.map do |token_start, token_end, statement_indent, type, parent|
                    "[#{token_start},#{token_end},#{statement_indent}#{type ? type.name : nil},#{parent.inspect}]"
                end.join(", ")
            end

            def size
                terms.size
            end

            def append(token_start, token_end, statement_indent, type, parent=nil)
                terms << [token_start, token_end, statement_indent, type, parent]
                self[-1]
            end

            def insert(index, token_start, token_end, statement_indent, type, parent=nil)
                terms.insert(index, [token_start, token_end, statement_indent, type, parent])
                self[index]
            end

            def each
                return enum_for(:each) unless block_given?
                0.upto(size-1).each { |index| yield Term.new(self, index) }
            end

            def [](index)
                index = terms.size + index if index < 0
                return nil if index >= terms.size
                Term.new(self, index)
            end

            def root
                root = self[0]
                return nil unless root
                root = root.parent while root.parent
                root
            end

            def source_range(line_data)
                SourceRange.new(line_data, 0, size > 0 ? 0 : self[-1].end)
            end

            def string(index)
                source_range(index).string
            end
        end
    end
end
