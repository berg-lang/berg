module BergLang
    class Parser
        # Represents the properties of a particular operator.
        # There may be more than one operator definition for a single "operator":
        # for example, "-" has two operator definitions, one for infix (3 - 2) and one for prefix (-1)
        class OperatorDefinition

            # "*", "-", etc. (nil if you do not want it to be matched)
            attr_reader :string

            # "*", "-", etc. (only different for "call")
            attr_reader :key

            # :infix, :prefix, :postfix, :start_delimiter, :end_delimiter
            attr_reader :type

            # 1-n, tightest-loosest
            attr_reader :precedence

            # :left, :right
            attr_reader :direction

            # If this is an end delimiter, started_by is the "key" value of the corresponding start delimiter.
            attr_reader :started_by

            # If this is an end delimiter, started_by is the "key" value of the corresponding start delimiter.
            attr_reader :ended_by

            # If this is true, and the line has only indentation, this opens an indented block,
            # terminated by the first non-whitespace line with *less* indentation than the current line.
            def opens_indent_block?
                @opens_indent_block
            end

            def initialize(string: nil, key: string, type: :infix, precedence:, direction: :left, started_by: nil, ended_by: nil, opens_indent_block: nil)
                if type == :indent
                    type = :infix
                    opens_indent_block = true
                end
                @string = string
                @key = key
                @type = type
                @precedence = precedence
                @direction = direction
                @started_by = started_by
                @ended_by = ended_by
                @opens_indent_block = opens_indent_block
            end

            def to_s
                if key.is_a?(Symbol)
                    key.inspect
                else
                    key
                end
            end

            def can_have_left_child?(left_operator)
                left_operator.precedence < precedence ||
                    (left_operator.precedence == precedence && direction == :left)
            end

            def started_by?(left_operator)
                left_operator.key == started_by
            end

            def start_delimiter?
                type == :start_delimiter
            end

            def end_delimiter?
                type == :end_delimiter
            end

            def prefix?
                type == :prefix || type == :start_delimiter
            end

            def infix?
                type == :infix
            end

            def postfix?
                type == :postfix || type == :end_delimiter
            end
        end
    end
end