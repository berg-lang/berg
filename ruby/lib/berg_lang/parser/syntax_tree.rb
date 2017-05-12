require_relative "expression"

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
            attr_reader :terms

            def initialize(source)
                @source = source
                @terms = []
            end

            include Enumerable

            def size
                terms.size
            end

            def append(token_start, token_end, token_type=nil, parent=nil)
                terms << [token_start, token_end, token_type, parent]
                index
            end

            def insert(index, token_start, token_end, token_type=nil)
                terms.insert(index, [token_start, token_end, token_type, parent])
                index
            end

            def each
                return enum_for(:each) unless block_given?
                0.upto(size-1).each { |index| yield Term.new(self, index) }
            end

            def [](index)
                index = terms.size + index if index < 0
                return nil if index >= terms.size
                SyntaxNode.new(self, index)
            end

            def source_range
                SourceRange.new(source, 0, self[-1].end)
            end

            def string(index)
                source_range(index).string
            end
        end
    end
end
