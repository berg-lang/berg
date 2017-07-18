require_relative "grammar"

module BergLang
    class Parser
        class ExpressionGrammar < Grammar
            def initialize(parser)
                super
                tokens
            end

            token_alias :bareword
            token_alias :string_literal
            token_alias :integer_literal, :float_literal, :hexadecimal_literal, :octal_literal, :imaginary_literal
            token_alias :whitespace, :comment, :insert, :eof, :sof
            token_alias :newline

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
                        { name: :empty, token_name: :insert, type: :expression },
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
                        { string: ":",   direction: :right, declaration: true, indented_variant_name: ": (indented)" },
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
                    [ { name: :apply, token_name: :insert } ],
                    [ { string: ";", statement_boundary: :end } ],
                    [ { name: :apply_block, token_name: :newline, space: :newline, significant: true, indented_variant_name: :apply }],
                    [
                        { name: :whitespace, type: :prefix, space: :whitespace },
                        { name: :whitespace, type: :postfix, space: :whitespace },
                        { name: :newline, type: :prefix, space: :newline },
                        { name: :newline, type: :postfix, space: :newline },
                        { name: :comment, type: :prefix, space: :comment },
                        { name: :comment, type: :postfix, space: :comment },
                        { name: :noinsert, token_name: :insert, type: :prefix, significant: false },
                        { name: :noinsert, token_name: :insert, type: :postfix, significant: false },
                    ],
                    # Delimiters want everything as children.
                    [
                        { string: "(",  type: :open,  closed_by: ")",       direction: :right, statement_boundary: :start },
                        { string: ")",  type: :close, opened_by: "(",     direction: :right, statement_boundary: :end },
                        { string: "{",  type: :open,  closed_by: "}",       direction: :right, statement_boundary: :start },
                        { string: "}",  type: :close, opened_by: "{",     direction: :right, statement_boundary: :end },
                        { name: :sof,    type: :open,  closed_by: :eof,      direction: :right, space: :newline, significant: true, statement_boundary: :start },
                        { name: :eof,    type: :close, opened_by: :sof,    direction: :right, space: :newline, significant: true, statement_boundary: :end },
                    ],
               )
            end
        end
    end
end