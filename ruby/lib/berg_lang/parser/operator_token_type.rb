require_relative "token_type"

module BergLang
    class Parser
        class OperatorTokenType < TokenType
            attr_reader :left
            attr_reader :right
            attr_accessor :started_by
            attr_accessor :ended_by

            def initialize(name, string: name, left:, right:, started_by: nil, ended_by: nil)
                raise "opens_indent_block unsupported on the left side" if left.opens_indent_block
                expression = self if left.expression? && right.expression?
                prefix = self if left.expression? && right.operator?
                postfix = self if left.operator? && right.expression?
                infix = self if left.operator? && right.operator?
                super(name, string: string, expression: expression, prefix: prefix, postfix: postfix, infix: infix)
                @string = name
                @left = left
                @right = right
                @started_by = started_by
                @ended_by = ended_by
            end
        end
    end
end
