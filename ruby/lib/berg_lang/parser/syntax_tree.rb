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
            attr_reader :terms
            attr_reader :line_locations

            def initialize(source)
                @source = source
                @terms = []
                @line_locations = []
            end

            include Enumerable

            def to_s
                terms.map do |term_start, term_end, type, parent|
                    "[#{term_start},#{term_end},#{type ? type.name : nil},#{parent.inspect}]"
                end.join(", ")
            end

            def size
                terms.size
            end

            def append(term_start, term_end, term_type=nil, parent=nil)
                terms << [term_start, term_end, term_type, parent]
                self[-1]
            end

            def insert(index, term_start, term_end, type=nil, parent=nil)
                terms.insert(index, [term_start, term_end, type, parent])
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

            def source_range
                SourceRange.new(self, 0, size > 0 ? 0 : self[-1].end)
            end

            def append_line(line_start, indent_size)
                line_locations << [line_start, indent_size]
            end

            def line_for(source_index)
                return nil if line_locations.empty?
                if line_locations.size > 1
                    line = line_locations.size * source_index / line_locations[-1][0]
                else
                    line = 0
                end
                line -= 1 while line_locations[line][0] > source_index
                line += 1 while line_locations[line][0] < source_index
                line + 1
            end

            def location_for(source_index)
                line = line_for(source_index)
                column = source_index - line_locations[line-1][0] + 1
            end

            def index_for(line)
                raise "invalid line number #{line}" if line < 1 || line > line_locations.size
                line_locations[line - 1][0]
            end

            def string(index)
                source_range(index).string
            end
        end
    end
end
