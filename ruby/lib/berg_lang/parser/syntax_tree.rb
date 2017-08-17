require_relative "../source/source_data"
require_relative "operation"
require_realtive "error"
module BergLang
    #
    # Represents the entire syntax tree of a given Source.
    #
    # The set of tokens is an array with parent pointers to indicate expression
    # children. Tokens are in the same order as the source, and are complete--anything
    # between one term's end and the next term's start is insignificant whitespace.
    #
    # Tree traversal is done by looking at the parent index of each term. If the parent
    # is to the left, it is a right operand of its parent; if it is to the right, it is
    # a left operand.
    #
    # Because the tokens are stored in lexical order, one's left operand is always one
    # of the parents of the previous term; and one's right operand is always one of the
    # parents of the next term.
    #
    class SyntaxTree
        #
        # Data about the actual source text.
        #
        attr_reader :source_data

        #
        # The list of tokens (used as operations).
        #
        # @return [Array[Token]] An array of tokens.
        #
        attr_reader :tokens

        #
        # The list of errors.
        #
        attr_reader :errors

        #
        # The list of comments, with source locations and attached syntax node index.
        #
        attr_reader :comment_data

        def initialize(source)
            @source_data = SourceData.new(source)
            @comment_data = []
            @errors = []
            @source_lines = []
            @tokens = []
        end

        include Enumerable

        #
        # The source of this syntax_tree
        #
        def source
            source_data.source
        end

        def report_error(error_type, range)
            error_data << Error.new(error_type, source_data.source_range(range))
        end

        def to_s
            tokens.map do |token|
                "[#{token.string.inspect} (#{token.index}#{token.string.size ? "-#{token.index+token.string.size}" : ""}) #{token.type ? token.type.name : nil}, parent=#{token.parent.string}]"
            end.join(", ")
        end

        #
        # Get the root of the tree.
        #
        def root
            root = first_token
            return nil unless root
            while root.parent_operator
                root = root.parent_operator
            end
            root
        end

        #
        # Get the number of tokens.
        #
        def size
            tokens.size
        end

        #
        # Get the first token (lexically).
        #
        def first_token
            self[0]
        end

        #
        # Get the last token (lexically).
        #
        def last_token
            self[-1]
        end

        #
        # Append a token.
        #
        def append(token_type, index, string, parent=nil)
            tokens << [token_type, index, string, parent]
            self[-1]
        end

        #
        # Enumerate tokens.
        #
        def each
            return enum_for(:each) unless block_given?
            0.upto(size-1).each { |index| yield Operation.new(self, index) }
        end

        #
        # Get the nth token.
        #
        def [](index)
            return nil if tokens.empty?
            index = tokens.size + index if index < 0
            return nil if index >= tokens.size
            Operation.new(self, index)
        end
    end
end
