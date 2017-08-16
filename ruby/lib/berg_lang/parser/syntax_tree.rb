require_relative "term"
require_relative "source_data"

module BergLang
    #
    # Represents the entire syntax tree of a given Source.
    #
    # The set of nodes is an array with parent pointers to indicate expression children.
    # nodes are in the same order as the source, and are complete--anything between one term's
    # end and the next term's start is insignificant whitespace.
    #
    # Tree traversal is done by looking at the parent index of each term. If the parent is to the
    # left, it is a right operand of its parent; if it is to the right, it is a left operand.
    #
    # Because the nodes are stored in lexical order, one's left operand is always one of the parents
    # of the previous term; and one's right operand is always one of the parents of the next term.
    #
    class SyntaxTree
        attr_reader :source_data
        attr_reader :comments
        attr_reader :nodes

        def initialize(source_data)
            @source_data = source_data
            @comments = {}
            @nodes = []
        end

        include Enumerable

        def source
            source_data.source
        end

        def to_s
            nodes.map do |token_type, index, string, parent|
                "[#{string.inspect} (#{index}#{string.size ? "-#{index+string.size}" : ""}) #{token_type ? token_type.name : nil}, parent=#{parent.}]"
            end.join(", ")
        end

        def size
            nodes.size
        end

        def root
            root = first_token
            return nil unless root
            while root.parent_operator
                root = root.parent_operator
            end
            root
        end

        def first_token
            self[0]
        end

        def last_token
            self[-1]
        end

        def append(token_type, index, string, parent=nil)
            nodes << [token_type, index, string, parent]
            self[-1]
        end

        def each
            return enum_for(:each) unless block_given?
            0.upto(size-1).each { |index| yield SyntaxNode.new(self, index) }
        end

        def [](index)
            return nil if nodes.empty?
            index = nodes.size + index if index < 0
            return nil if index >= nodes.size
            SyntaxNode.new(self, index)
        end
    end
end
