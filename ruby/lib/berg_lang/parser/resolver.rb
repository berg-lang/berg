require_relative "tokenizer"

module BergLang
    class Parser
        #
        # Reads and disambiguates operator and expression tokens from the source.
        #
        # Handles operator disambiguation, indent/undent, newline as operator, sticky prefix/postfix, apply (a b
        # is a function call), and empty (e.g. a: 1, b:, c: has empty for b and c).
        #
        # Does not handle precedence and parent hookup.
        #
        class Resolver
            attr_reader :parser
            attr_reader :tokenizer
            attr_reader :syntax_tree

            def initialize(parser)
                @parser = parser
                @tokenizer = Tokenizer.new(parser)
                @syntax_tree = SyntaxTree.new
                @open_indents = []
            end

            #
            # Parses, resolves, and creates a syntax tree for a source.
            #
            def parse
                while parse_next
                end
            end

            private

            #
            # Open indents
            #
            attr_reader :open_indents

            #
            # Parses, resolves, and associates a set of operator/expression tokens.
            #
            # May parse more than one token, if there is ambiguity (operators like + and - can be prefix/postfix
            # as well as infix).
            #
            # @return [true,false] `true` if something was parsed, `false` if parsing is complete. 
            #
            def parse_next
                last_token = syntax_tree[-1]
                prefer_operator = last_token.type.right.needs_operator? if last_token
                index = resolve_expression_sequence(prefer_operator)
                if index
                    associate_operators(syntax_tree[index])
                    true
                else
                    false
                end
            end

            #
            # Associates the operators by setting their parents correctly (according to precedence).
            #
            # Will handle all nodes starting from the given node to the end of the tree.
            #
            # @param [SyntaxNode] The first node in the syntax tree to associate.
            #
            def associate_operators(node)
                begin
                    associate_operator(node)
                end while node = node.next_term
            end

            #
            # Associates a single operator with the rest of the tree by setting its parent and child correctly.
            #
            # @param [SyntaxNode] The node in the tree to associate.
            #
            def associate_operator(node)
                parent = node.previous_term
                left_side = node.type.left
                if left_side.is_operator?
                    while parent && left_side.can_have_child?(parent.type)
                        left_child = parent
                        parent = left_child.parent
                    end
                    if !left_child
                        raise syntax_errors.internal_error(node, "#{node} cannot have left child #{parent}!")
                    end
                    left_child.parent = node
                end
                if !parent.type.right.can_have_child?(node.type)
                    raise syntax_errors.internal_error(node, "#{node} cannot have parent #{parent}!")
                end
                node.parent = parent
            end

            #
            # Reads the next token from the tokenizer, undenting if necessary
            #
            # @return [Token] the next token from the tokenizer
            #
            def read_token
                token = @token || tokenizer.read_token
                # Handle undent
                if (@token || token.leading_newline?) && open_indents.any?
                    if token.indent_size < open_indents[-1]
                        open_indents.pop
                        undent_token = token.dup
                        undent_token.type = parser.types.undent
                        undent_token.end = undent_token.start
                        undent_token.trailing_space = undent_token.end
                        token.leading_space = undent_token.start
                        token.leading_newline = nil
                        @token = token
                        return undent_token
                    end
                end
                @token = nil
                token
            end

            #
            # Parses and resolves tokens to form a valid expression sequence.
            #
            # May parse more than one token, if there is ambiguity (operators like + and - can be prefix/postfix
            # as well as infix).
            #
            # Will avoid treating newline as an operator, or inserting call / empty, as long as possible.
            #
            # @param [true,false] prefer_operator `true` if we should be infix/postfix (i.e. take an operand on the
            #        left side), `false` if we should be expression/prefix (no operand on the left side).
            #
            # @return [true,false] `true` if we are at EOF and could not do any work, `false` otherwise.
            #
            def resolve_expression_sequence(prefer_operator)
                token = read_token
                return nil unless token
                index, next_type = process_token(token, prefer_operator, allow_newline_operator: true)

                # If we have to insert call or empty to make it work, do that.
                if next_type.left.is_operator?
                    insert_empty(index) if !prefer_operator
                else
                    insert_call(index) if prefer_operator
                end

                index
            end

            private

            #
            # Process a single token in an expression sequence, disambiguating it so that the expression sequence
            # is valid.
            #
            # @param [Token] token The token to process.
            # @param [true,false] prefer_operator `true` if we should be infix/postfix (i.e. take an operand on
            #        the left side), `false` if we should be expression/prefix (no operand on the left side).
            # @param [true,false] allow_newline_operator `true` if we should insert a newline "statement separator"
            #        infix operator if we can't be an operator and we have a newline we can use.
            #
            # @return [Integer] The index of the token in the tree.
            #
            def process_token(token, prefer_operator, allow_newline_operator)
                # Add the token to the syntax tree so we can work with it.
                index = syntax_tree.append(start: token.start, end: token.end)

                # If we're asked for operator, but will end up being expression / prefix, we'll end up using a
                # newline.
                type = choose_token_type(token, prefer_operator, allow_newline_operator)
                syntax_tree[index].type = type
                if type.right.opens_indent_block? && type.trailing_newline?
                    insert_indent(index+1)
                end

                if prefer_operator && !type.left.is_operator?
                    return insert_newline(index, token) if allow_newline_operator && token.leading_newline?
                end
                [ index, type ]
            end

            #
            # Decide a token's type.
            #
            # We use our token type and the preference of the previous token to decide what we should be.
            #
            # If the left token type is not enough to narrow the choices down to one, we call
            # choose_ambiguous_token_type to handle the ambiguity.
            #
            # @param [Token] token The token to process.
            # @param [true,false] prefer_operator `true` if we should be infix/postfix (i.e. take an operand on
            #        the left side), `false` if we should be expression/prefix (no operand on the left side).
            # @param [true,false] allow_newline_operator `true` if we should insert a newline "statement separator"
            #        infix operator if we can't be an operator and we have a newline we can use.
            #
            # @return TokenType the token type we chose.
            #
            def choose_token_type(token, prefer_operator, allow_newline_operator)
                # Handle sticky expressions: treat infix as if it weren't there if we are sticky postfix or
                # prefix. There is no case in which we will pick infix over prefix or postfix in these situations.
                token.infix = nil if token.leading_space? && !token.trailing_space? && token.prefix?
                token.infix = nil if !token.leading_space? && token.trailing_space? && token.postfix?

                # If our right side is unambiguous, we can decide our left side right now.
                if token.right_is_operator? == true
                    return token.prefix if prefer_operator == false || !token.infix?
                    return token.infix
                elsif token.right_is_operator? == false
                    return token.postfix if prefer_operator == true || !token.expression?
                    return token.expression
                end

                choose_ambiguous_token_type(token, prefer_operator, allow_newline_operator)
            end

            #
            # Decide a token's type when there are multiple possibilities (infix or prefix, AND expression or postfix).
            #
            # We use the preference of the token on the left to decide what we should be.
            #
            # If the left token type is not enough to narrow the choices down to one, we parse the next
            # token to narrow the field further, setting prefer_operator such that the next token will
            # try to pick something that will honor *our* preference. (This may be the same as, or different from,
            # our own prefer operator).
            #
            # If after parsing the token and taking the previous token's type into account, we don't have an
            # answer, we pick infix over prefix, and expression over postfix.  We don't have to choose between
            # expression/infix or prefix/postfix at this point, because the right side will be either an expression
            # (in which case we prefer infix or prefix) or an operator (in which case we prefer expression or postfix).
            #
            # @param [Token] token The token to process.
            # @param [true,false] prefer_operator `true` if we should be infix/postfix (i.e. take an operand on
            #        the left side), `false` if we should be expression/prefix (no operand on the left side).
            # @param [true,false] allow_newline_operator `true` if we should insert a newline "statement separator"
            #        infix operator if we can't be an operator and we have a newline we can use.
            #
            # @return TokenType the token type we chose.
            #
            def choose_ambiguous_token_type(token, prefer_operator, allow_newline_operator)
                if prefer_operator
                    # If we can pick newline, don't let the next guy pick it (do it earlier rather than later).
                    if allow_newline_operator && token.leading_newline?
                        if (token.prefix && !token.prefix) || (token.expression && !token.postfix)
                            allow_newline_operator = false
                        end
                    end
                    preferred = token.infix || token.postfix || token.prefix || token.expression
                else
                    preferred = token.expression || token.prefix || token.infix || token.postfix
                end

                # Read the next token.
                next_token = read_token
                next_index, next_type = process_token(next_token, preferred.right.is_operator?, allow_newline_operator)

                # Choose infix or prefix if the right side prefers we are an operator. Otherwise, expression/postfix.
                if next_type.left.needs_operator?
                    return prefer_operator ? token.infix || token.prefix : token.prefix || token.infix
                else
                    return prefer_operator ? token.postfix || token.expression : token.expression || token.postfix
                end
            end

            def insert_indent(index, token)
                open_indents << token.indent_size
                type = parser.tokens.indent
                syntax_tree.insert(index, token.end, token.end, type)
            end

            def insert_newline(index, token)
                type = parser.tokens.newline_operator
                syntax_tree.insert(index, token.leading_newline, token.leading_newline, type)
                [ index, type ]
            end

            def insert_apply(index, token)
                syntax_tree.insert(index, token.leading_space, token.leading_space, parser.tokens.apply_operator)
            end

            def insert_empty(index, token)
                syntax_tree.insert(index, token.leading_space, token.leading_space, parser.tokens.empty)
            end
        end
    end
end
