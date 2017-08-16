require_relative "syntax_tree_builder"

module BergLang
    module Parser
        class Resolver
            attr_reader :syntax_tree_builder
            attr_reader :unresolved_symbols

            def initialize(syntax_tree_builder, output)
                @syntax_tree_builder = syntax_tree_builder
                @output = output
            end

            def append_symbol(symbol_type, range, string)
                case symbol_type
                when grammar.newline
                    handle_newline(range)
                when grammar.indent
                    handle_indent(range, string)
                when grammar.space
                    # Do nothing with space.
                else
                    resolve_symbol(symbol_type, range, string)
                end
            end

            def close
                resolve_all(as_noun: true)
            end

            private

            def handle_indent(range, string)
                #
                # On indent:
                # - Error if indent and previous indent do not match
                # - Handle undent.
                #   - If indent <= parent margin:
                #     - Resolve all as noun
                #   - If resolved verb, append empty_block/missing_operand
                #   - If resolved, mark visible indent
                #

                if last_indent
                    case last_indent.size <=> string.size
                    when 0
                        match = last_indent == string
                    when 1
                        match = last_indent.substr(string.size) == string
                    when -1
                        match = string.substr(last_indent.size) == last_indent
                    end

                    if !match
                        # TODO add other indent range as well for comparison!
                        append_ambiguous(grammar.error_non_matching_indent, range, string)
                    end
                end

                @last_indent_range = range
                @last_indent = string
            end

            def resolve_symbol(symbol_type, range, string)
                noun, verb = preferred_token_types(symbol_type)
                if noun && verb
                    append_ambiguous(noun, verb, range, string)
                else
                    # Unambiguous
                    # - resolve all accordingly
                    # - append token
                    # - Prefer operator/operand accordingly
                    token_type = noun || verb
                    is_operator = [ :infix, :prefix ].include?(token_type.operator_type)
                    resolve_all(is_operator)
                    append_token(token_type, range, string, leading_indent_range)
                    @prefer_operator = !!noun
                end
            end

            def preferred_token_types(symbol_type)
                # If leading space only, prefer operand for this purposes.
                if !prefer_operator || (leading_space? && !trailing_space?)
                    # If prefer operand, prefix > infix and expression > prefix
                    noun = symbol_type.suffix || symbol_type.expression
                    verb = symbol_type.infix  || symbol_type.prefix
                else
                    # If prefer operator, infix > prefix and postfix > expression
                    noun = symbol_type.expression || symbol_type.suffix
                    verb = symbol_type.prefix     || symbol_type.infix
                end
                # If trailing space only, exclude prefix/infix if possible
                verb = nil if noun && verb && !leading_space? && trailing_space?
                [ noun, verb ]
            end

            def handle_newline(symbol_start, symbol_end)
                # On newline:
                #   - Add to source_data
                #   - If unresolved and no insert_newline, resolve all as noun
                #   - If resolved noun, set insert_newline
                source_data.append_line(symbol_end)
                if !resolved? && !insert_newline
                    resolve_all(as_noun: true)
                end
                if resolved? && resolved_as_noun?
                    @insert_newline_start = symbol_start
                    @insert_newline_end = symbol_end
                end
            end

            def append_ambiguous(noun, verb, range, string)
                #     - resolve all accordingly
                #     - append ambiguous
                #     - Prefer operator/operand accordingly
                #   - If double ambiguous:
                #     - append ambiguous
                #     - If prefix/postfix, keep preference
                #     - If infix/expression, flip preference

                # If ambiguous on right side only, resolve all accordingly.
                case [ noun.operation_type, verb.operation_type ]
                when [ :expression, :prefix ]
                    resolve_all(as_noun: false)
                when [ :postfix, :infix ]
                    resolve_all(as_noun: true)

                else

                    # Honor apply > empty: If left-side ambiguous, and prefer operator, and unresolved inserts error, resolve all as noun (no empty insert)
                    if prefer_operator && ambiguous? && !resolved_as_noun?
                        resolve_all(as_noun: true)
                    end
                end

                unresolved_symbols << UnresolvedSymbol.new(noun, verb, range, string, leading_indent_range)

                case [ noun.operation_type, verb.operation_type ]
                when [ :expression, :prefix ]
                    @prefer_operator = true
                when [ :postfix, :infix ]
                    @prefer_operator = false
                # expression/infix: flip preference
                when [ :expression, :infix ]
                    @prefer_operator = !@prefer_operator
                # postfix/prefix: keep preference the same
                when [ :postfix, :prefix ]
                else
                    raise "Unexpected noun/verb combo: #{noun.operation_type}/#{verb.operation_type}"
                end
            end

            def append_token(token_type, range, string, leading_indent_range)
                syntax_tree_builder.append(token_type, range, string, leading_indent_range)
            end

            def resolve_all(as_noun:)
                # Insert non-match: If as_noun != prefer_operator
                if as_noun != prefer_operator
                    # - If last_resolved_token is expression/postfix:
                    #   - If insert_newline:
                    #     - If first unresolved indent > margin, append apply
                    #     - Else append extend
                    #   - Else append apply
                    # - If last_resolved_token is prefix/infix:
                    #   - Append empty (last_resolved_token.missing_parameter_symbol)
                    #
                    if resolved_as_noun?
                        if insert_newline_range
                            # If insert_newline, and we're indented, append apply
                            # Else append extend
                            if unresolved_tokens.first.leading_indent.size > syntax_tree_builder.margin
                                insert_type = grammar.apply
                            else
                                insert_type = grammar.extend
                            end
                        else
                            insert_type = grammar.apply
                        end

                    else
                        # Append empty
                        if as_noun
                            if last_resolved_token
                                insert_type = last_resolved_token.empty_parameter_symbol
                            else
                                insert_type = grammar.empty_block
                            end

                        # Honor apply > empty: If we are being asked to resolve
                        # as a verb, resolve as noun (don't append empty) and
                        # then append apply to make up the difference.
                        else
                            resolve_all(as_noun: true)
                            insert_type = grammar.apply
                        end
                    end

                    insert_position = last_resolved_token.range.begin
                    insert_range = insert_position...insert_position
                    append_token(insert_type, insert_range, "", nil)
                end

                #
                # Append all:
                # - For each unresolved:
                #   - Append verb if last_resolved_token is expression/postfix
                #   - Else append noun
                #
                unresolved_symbols.each do |unresolved_symbol|
                    token_type = resolved_as_noun? ? unresolved_symbol.verb : unresolved_symbol.noun
                    syntax_tree_builder.append_token(token_type, unresolved_symbol.range, unresolved_symbol.string, unresolved_symbol.leading_indent)
                end
            end
        end
    end
end
