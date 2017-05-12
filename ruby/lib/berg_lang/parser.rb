require_relative "output"
require_relative "berg_tokens"
require_relative "parser/syntax_errors"
require_relative "parser/resolver"

module BergLang
    #
    # Parses Berg.
    #
    class Parser
        attr_reader :source
        attr_reader :output

        def initialize(source, output: Output.new(STDOUT))
            @source = source
            @output = output
            @resolver = Resolver.new(self)
        end

        def tokens
            BergTokens
        end

        def parse
            # Go through each token, find its left parent or child as you go, and set it.
            # If it could have a left parent OR a left child (is ambiguous), move to the next one.
            prefer_operator = false
            while prefer_operator = resolver.parse_expression_sequence(prefer_operator)
                set_parents
            end
        end

        private


        # Sets the parents/children of everything in ast from index on.
        def build_tree(index)
            if ast[index].left_is == :operator
                if ast[index-1].right_is != :expression
                    insert_call_or_newline()

        end

        def source_range
            unclosed.source_range
        end

        private

        attr_reader :tokenizer
        attr_reader :unclosed_expression
        attr_reader :token
        attr_reader :previous_token
        attr_reader :current_indent

        def handle_bare_declaration(operator)
            if operator.key == ":" && !previous_token.is_a?(Ast::Bareword)
                if next_token.is_a?(Ast::Bareword)
                    Ast::PrefixOperation.new(advance_token, advance_token)
                end
            end
        end

        def check_for_dot_number!(operator)
            if operator.key == "." && next_token.is_a?(Ast::NumericLiteral)
                raise syntax_errors.float_without_leading_zero(SourceRange.span(operator, next_token))
            end
        end

        def handle_indent(operator, operators, indent)
            #
            # Handle open indent: if we see a : operator followed by \n, insert an open indent before the whitespace comes around.
            # possible for the next expression to have a *smaller* indent, in which case an undent and empty expression will happen.
            #
            if operator.opens_indent_block? && token.is_a?(Ast::Whitespace) && token.newline
                output.debug("Indent: #{operator} followed by newline. Current indent is #{indent.string.inspect}.")
                indent_start = source.create_empty_range(operator.source_range.end)
                open_indent = Ast::IndentOperator.new(indent_start, indent, all_operators[:indent][:open])
                operators << open_indent
            end
        end

        def handle_undent(whitespace, operators)
            open_indents = unclosed_expression.open_indents
            open_indents = open_indents + operators.select { |operator| operator.is_a?(Ast::IndentOperator) }
            open_indents.reverse_each do |open_indent|
                # Truncate both indents and make sure they match as far as tabs/spaces go
                if open_indent.indent.string[0...whitespace.indent.size] != whitespace.indent.string[0...open_indent.indent.size]
                    raise syntax_errors.unmatchable_indent(whitespace.indent, open_indent.indent)
                end

                # If we're properly indented, we won't find any further smaller indents. Exit early.
                break if whitespace.indent.size > open_indent.indent.size

                output.debug("Undent: #{whitespace.indent.string.inspect} followed by newline")
                undent = Ast::Operator.new(source.create_empty_range(whitespace.indent.end), all_operators[:undent][:close])
                empty_expression = handle_empty_block(undent, operators)
                return empty_expression if empty_expression

                operators << undent
            end
            nil
        end
    end
end
