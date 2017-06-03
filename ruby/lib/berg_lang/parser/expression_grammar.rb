require_relative "grammar"

module BergLang
    class Parser
        class ExpressionGrammar < Grammar
            def initialize(parser)
                super
                tokens
            end

            term_alias :bareword
            term_alias :string_literal
            term_alias :integer_literal, :float_literal, :hexadecimal_literal, :octal_literal, :imaginary_literal
            term_alias :whitespace, :newline, :comment, :border, :eof, :sof, :indent, :undent

            #
            # tokens
            #

            def tokens
                @tokens ||= define_tokens(
                    [
                        { name: :bareword, type: :expression },
                        { name: :string_literal, type: :expression },
                        { name: :integer_literal, type: :expression },
                        { name: :float_literal, type: :expression },
                        { name: :hexadecimal_literal, type: :expression },
                        { name: :octal_literal, type: :expression },
                        { name: :empty, token_name: :border, type: :expression },
                    ],
                    ". postfix.-- postfix.++",
                    "right prefix.-- prefix.++ prefix.- prefix.+ prefix.!",
                    "* / %",
                    "+ -",
                    "> >= < <=",
                    "== !=",
                    "postfix.+ postfix.* postfix.?",
                    "&&",
                    "|| ??",
                    "->",
                    [
                        { string: ":",   direction: :right, declaration: true, opens_indent_block: true, },
                        { string: "=",   direction: :right, declaration: true, },
                        { string: "+=",  direction: :right, declaration: true, },
                        { string: "-=",  direction: :right, declaration: true, },
                        { string: "*=",  direction: :right, declaration: true, },
                        { string: "/=",  direction: :right, declaration: true, },
                        { string: "%=",  direction: :right, declaration: true, },
                        { string: "||=", direction: :right, declaration: true, },
                        { string: "&&=", direction: :right, declaration: true, },
                        { string: "??=", direction: :right, declaration: true, },
                    ],
                    [ "," ],
                    # TODO unsure if this is the right spot for intersect/union. Maybe closer to - and +
                    "&",
                    "|",
                    [ { name: :apply, token_name: :border } ],
                    [ ";", { name: :newline } ],
                    [
                        { name: :whitespace, type: :prefix, space: true },
                        { name: :whitespace, type: :postfix, space: true },
                        { name: :newline, type: :prefix, space: true },
                        { name: :newline, type: :postfix, space: true },
                        { name: :comment, type: :prefix, space: true },
                        { name: :comment, type: :postfix, space: true },
                        { name: :border, type: :prefix, space: true },
                        { name: :border, type: :postfix, space: true },
                    ],
                    # Delimiters want everything as children.
                    [
                        { name: :indent, type: :open,  closed_by: :undent,   direction: :right },
                        { name: :undent, type: :close, opened_by: :indent, direction: :right },
                        { string: "(",  type: :open,  closed_by: ")",       direction: :right },
                        { string: ")",  type: :close, opened_by: "(",     direction: :right },
                        { string: "{",  type: :open,  closed_by: "}",       direction: :right },
                        { string: "}",  type: :close, opened_by: "{",     direction: :right },
                        { name: :sof,    type: :open,  closed_by: :eof,      direction: :right, space: true },
                        { name: :eof,    type: :close, opened_by: :sof,    direction: :right, space: true },
                    ],
               )
            end
        end
    end
end