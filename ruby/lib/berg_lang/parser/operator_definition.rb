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

            # :infix, :prefix, :postfix, :open, :close
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

            # If this is true, the left side binds as tightly as possible. A: b, a = b
            def declaration?
                @declaration
            end

            # If this is true, a -b and a+ b use the prefix and postfix form, respectively, regardless of any other concern.
            def can_be_sticky?
                @can_be_sticky
            end

            # If this is true, the operator is not allowed to be picked by the infix/prefix/postfix disambiguator.
            # Used for special cases like :bareword (bare parameter declaration)
            def resolve_manually?
                @resolve_manually
            end

            def initialize(string: nil, key: string, type: :infix, precedence:, direction: :left, started_by: nil, ended_by: nil, opens_indent_block: nil, declaration: nil, can_be_sticky: true, resolve_manually: nil)
                @string = string
                @key = key
                @type = type
                @precedence = precedence
                @direction = direction
                @started_by = started_by
                @ended_by = ended_by
                @opens_indent_block = opens_indent_block
                @can_be_sticky = can_be_sticky
                @resolve_manually = resolve_manually
            end

            def to_s
                if key.is_a?(Symbol)
                    key.inspect
                else
                    key
                end
            end

            def can_have_left_child?(left_operator)
                return false if declaration?
                return true if left_operator.precedence < precedence
                return true if left_operator.precedence == precedence && direction == :left
            end

            def started_by?(left_operator)
                left_operator.key == started_by
            end

            def open?
                type == :open
            end

            def close?
                type == :close
            end

            def prefix?
                type == :prefix || type == :open
            end

            def infix?
                type == :infix
            end

            def postfix?
                type == :postfix || type == :close
            end
        end
    end
end