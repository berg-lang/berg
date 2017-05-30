require_relative "term_type"
require_relative "term_type/ambiguous"
require_relative "term_type/term"
require_relative "term_type/filler"
require_relative "term_type/side"
require "set"

module BergLang
    class Parser
        module BergTerms
            def self.filler
                @filler ||= Set.new([whitespace, newline, comment])
            end

            def self.expressions
                @expressions ||= Set.new([
                    bare_declaration,
                    bareword,
                    string_literal,
                    hexadecimal_literal,
                    octal_literal,
                    integer_literal,
                    float_literal,
                    imaginary_literal,
                    empty
                ])
            end

            def self.whitespace
                @whitespace ||= define_filler("whitespace", whitespace: true)
            end

            def self.newline
                @newline ||= TermType::Ambiguous.new(filler: whitespace, infix: newline_operator)
            end

            def self.comment
                @comment ||= define_filler("comment")
            end

            def self.empty
                @empty ||= define_expression_term("empty")
            end

            def self.bare_declaration
                @bare_declaration ||= define_expression_term("bare_declaration")
            end

            def self.bareword
                @bareword ||= define_expression_term("bareword")
            end

            def self.string_literal
                @string_literal ||= define_expression_term("string literal")
            end

            def self.hexadecimal_literal
                @hexadecimal_literal ||= define_expression_term("hexadecimal literal")
            end

            def self.octal_literal
                @octal_literal ||= define_expression_term("octal literal")
            end

            def self.integer_literal
                @integer_literal ||= define_expression_term("integer literal")
            end

            def self.float_literal
                @float_literal ||= define_expression_term("float literal")
            end

            def self.imaginary_literal
                @imaginary_literal ||= define_expression_term("imaginary literal")
            end

            def self.eof
                operators[:eof]
            end

            def self.sof
                operators[:sof]
            end

            def self.apply
                operators[:apply]
            end

            def self.newline_operator
                operators[:newline]
            end

            def self.indent
                operators[:indent]
            end

            def self.undent
                operators[:undent]
            end

            def self.operators
                @operators ||= define_operator_groups(
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
                    [ { name: :apply } ],
                    [ ";", { name: :newline } ],
                    # Delimiters want everything as children.
                    [
                        { name: :indent, type: :open,  closed_by: :undent,   direction: :right },
                        { name: :undent, type: :close, opened_by: :indent, direction: :right },
                        { string: "(",  type: :open,  closed_by: ")",       direction: :right },
                        { string: ")",  type: :close, opened_by: "(",     direction: :right },
                        { string: "{",  type: :open,  closed_by: "}",       direction: :right },
                        { string: "}",  type: :close, opened_by: "{",     direction: :right },
                        { name: :sof,    type: :open,  closed_by: :eof,      direction: :right },
                        { name: :eof,    type: :close, opened_by: :sof,    direction: :right },
                    ],
               )
            end

            private

            def self.define_filler(name, whitespace:)
                TermType::Filler.new(name, whitespace: whitespace)
            end

            def self.define_expression_term(name)
                TermType::Term.new(name)
            end

            def self.define_operators(*operator_defs)
                #
                # Process the nice operator string definitions
                #
                direction = nil
                operators = operator_defs.flat_map do |operator_def|
                    if operator_def.is_a?(String)
                        # String is like "* / + *"
                        operator_def = operator_def.split(/ /)

                        # If string starts with "right", like "right = += -=", use that as direction
                        if %w{left right}.include?(operator_def.first)
                            direction ||= operator_def.shift.to_sym
                        else
                            direction ||= :left
                        end

                        # Parse through looking for prefix, infix, etc. "++.postfix --.postfix"
                        operator_def.map do |operator_string|
                            if operator_string =~ /^(.+)\.(.+)$/
                                define_operator(string: $2, type: $1.to_sym)
                            else
                                define_operator(string: operator_string)
                            end
                        end
                    else
                        direction ||= operator_def.delete(:direction)
                        define_operator(**operator_def)
                    end
                end
                [ direction, operators ]
            end

            def self.define_operator(string: nil, name: string, type: :infix, opened_by: nil, closed_by: nil, opens_indent_block: nil, declaration: nil, direction: nil)
                if [:infix, :postfix, :close ].include?(type)
                    left = TermType::Side.new(declaration: declaration, closed_by: closed_by)
                end
                if [:infix, :prefix, :open ].include?(type)
                    right = TermType::Side.new(opens_indent_block: opens_indent_block, opened_by: opened_by)
                end
                TermType::Term.new(name, string: string, left: left, right: right)
            end

            def self.define_operator_groups(*groups)
                operators = {}
                all_terms = expressions
                groups.each do |operator_defs|
                    operator_defs = Array(operator_defs).flatten
                    direction, operator_group = define_operators(*operator_defs)

                    # Handle precedence.
                    # The definition of precedence is that operators at level n are unwilling to have operators
                    # at level n+1 as children. As you go up, you only have lower levels as children.
                    # Operators at the same level are willing to have each other as left or right children,
                    # depending on direction.
                    left_terms = all_terms
                    left_terms = left_terms + operator_group if direction == :left
                    right_terms = all_terms
                    right_terms = right_terms + operator_group if direction != :left
                    operator_group.each do |operator|
                        # Operators from prior levels are defined earlier.
                        operator.left.accepts_operands += left_terms if operator.left
                        operator.right.accepts_operands += right_terms if operator.right

                        # Add the operators to the full group
                        all_terms << operator
                        if !operators.include?(operator.name)
                            operators[operator.name] = operator
                        else
                            operators[operator.name] += operator
                        end
                    end

                    all_terms += operator_group
                end

                # Handle open and close operator precedence a teensy bit differently:
                # - no one can have an open operator as a left child.
                # - no one can have a close operator as a right child.
                all_terms.each do |term|
                    if term.left
                        term.left.accepts_operands.reject! do |accepts|
                            if accepts.right && accepts.right.closed_by
                                accepts.right.accepts_operands << term
                                true
                            end
                        end
                    end
                    if term.right
                        term.right.accepts_operands.reject! do |accepts|
                            if accepts.left && accepts.left.opened_by
                                accepts.left.accepts_operands << term
                                true
                            end
                        end
                    end
                end

                # Sort result by precedence.
                operators.sort_by { |name, value| name.is_a?(String) ? -name.size : 0 }.to_h
            end
        end
    end
end