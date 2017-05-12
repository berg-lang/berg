require_relative "token_type"
require_relative "operator_token_type"

module BergLang
    class Parser
        module BergTokens
            def self.whitespace
                @whitespace ||= TokenType.new("whitespace", whitespace: true)
            end

            def self.newline
                @newline ||= TokenType.new("newline whitespace", whitespace: true, newline: true)
            end

            def self.comment
                @newline ||= TokenType.new("comment", whitespace: true)
            end

            def self.bareword
                @bareword ||= define_expression_token("bareword")
            end

            def self.string_literal
                @string_literal ||= define_expression_token("string literal")
            end

            def self.hexadecimal_literal
                @hexadecimal_literal ||= define_expression_token("hexadecimal literal")
            end

            def self.octal_literal
                @octal_literal ||= define_expression_token("octal literal")
            end

            def self.integer_literal
                @integer_literal ||= define_expression_token("integer literal")
            end

            def self.float_literal
                @float_literal ||= define_expression_token("float literal")
            end

            def self.imaginary_literal
                @imaginary_literal ||= define_expression_token("imaginary literal")
            end

            def self.eof
                operators[:eof]
            end

            def self.sof
                operators[:sof]
            end

            def self.call
                operators[:call]
            end

            def self.newline_operator
                operators["\n"]
            end

            def self.indent
                operators[:indent]
            end

            def self.undent
                operators[:undent]
            end

            def self.operators
                @operators ||= define_operator_groups(
                    [
                        # :BareDeclaration is handled outside the normal arity resolution rules because the rule is <!bareword>:bareword,
                        # which doesn't fit normal rules. Will have to think if there is a way ...
                        { string: ":", type: :prefix, resolve_manually: true },
                    ],
                    ". postfix.-- postfix.++",
                    "prefix.-- prefix.++ prefix.- prefix.+ prefix.!",
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
                    [ ",", { string: ",", type: :postfix, can_be_sticky: false } ],
                    # TODO unsure if this is the right spot for intersect/union. Maybe closer to - and +
                    "&",
                    "|",
                    [ { key: :call } ],
                    [ "\n ;", { string: ";", type: :postfix, can_be_sticky: false } ],
                    # Delimiters want everything as children.
                    [
                        { key: :indent, type: :open,  ended_by: :undent,   direction: :right },
                        { key: :undent, type: :close, started_by: :indent, direction: :right },
                        { string: "(",  type: :open,  ended_by: ")",       direction: :right },
                        { string: ")",  type: :close, started_by: "(",     direction: :right },
                        { string: "{",  type: :open,  ended_by: "}",       direction: :right },
                        { string: "}",  type: :close, started_by: "{",     direction: :right },
                        { key: :sof,    type: :open,  ended_by: :eof,      direction: :right },
                        { key: :eof,    type: :close, started_by: :sof,    direction: :right },
                    ],
                )
            end

            private

            def self.define_expression_token(name)
                OperatorTokenType.new(name, left: TokenSide.new(is_operator: false), right: TokenSide.new(is_operator: false))
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

            def self.define_operator(string: nil, key: string, type: :infix, started_by: nil, ended_by: nil, opens_indent_block: nil, declaration: nil, can_be_sticky: true, resolve_manually: nil)
                case type
                when :infix
                    left_is_operator, right_is_operator = [true, true]
                when :prefix, :open
                    left_is_operator, right_is_operator = [false, true]
                when :postfix, :close
                    left_is_operator, right_is_operator = [true, false]
                else
                    raise "Unexpected type #{type}!"
                end
                left = OperatorSide.new(is_operator: left_is_operator, can_be_sticky: can_be_sticky, resolve_manually: resolve_manually)
                right = OperatorSide.new(is_operator: right_is_operator, can_be_sticky: can_be_sticky, resolve_manually: resolve_manually, opens_indent_block: opens_indent_block)

                OperatorTokenType.new(string: string, key: key, left: left, right: right, started_by: started_by, ended_by: ended_by)
            end

            def self.define_operator_groups(*groups)
                operators = {}
                groups.each do |operator_defs|
                    operator_defs = Array(operator_defs).flatten
                    direction, operator_group = define_operators(*operator_defs)

                    # Handle precedence.
                    # The definition of precedence is that operators at level n are unwilling to have operators
                    # at level n+1 as children. As you go up, you only have lower levels as children.
                    # Operators at the same level are willing to have each other as left or right children,
                    # depending on direction.
                    operator_group.each do |operator|
                        # Operators from prior levels are defined earlier.
                        operator.left.accepts_children += operators.values.flat_map { |op| op.variants }
                        operator.right.accepts_children += operators.values.flat_map { |op| op.variants }
                        # Operators at the same precedence can have each other as left/right children only.
                        # We don't want a situation where operators have each other as children, that creates
                        # confusing resolution issues.
                        if direction == :left
                            operator.left.accepts_children += operator_group
                        else
                            operator.left.accepts_children += operator_group
                        end

                        operators[operator.key] ||= TokenType.new
                        operators[operator.key].infix = operator if operator.infix?
                        operators[operator.key].prefix = operator if operator.prefix?
                        operators[operator.key].postfix = operator if operator.postfix?
                    end

                    all_operators += operators

                    operator_defs.each do |operator|
                    end


                end.group_by { |operator| operator[:token_type].key }

                # Construct the actual ambiguous token type by rolling up the prefix/postfix/infix operator
                # for each key. For example, + has postfix prefix and infix, each with different precedences.
                # We group them together so that the parser can see them all at once and make a decision.
                operators = operators.map do |key, token_types|
                    token_types.each do |op|
                        token_type = op[:token_type]
                        infix = token_type if token_type.infix?
                        prefix = token_type if token_type.prefix?
                        postfix = token_type if token_type.postfix?
                    end
                    [ key, TokenType.new(infix: infix, prefix: prefix, postfix: postfix) ]
                end

                operators.each_value do |operator|
                    operator.
                end

                # Attach operators to each other according to precedence.
                operators.sort_by { |key, value| key.is_a?(String) ? -key.size : 0 }.to_h
            end
        end
    end
end