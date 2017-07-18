module BergLang
    class Parser
        class ParseState
            attr_reader :parse_result

            #
            # The scanner that will be used for the next token.
            #
            attr_reader :scanner
            attr_reader :prev_is_space
            attr_reader :current_indent
            attr_reader :prefer_operand_next

            attr_reader :if_operand_next
            attr_reader :statement_indent_if_operand_next
            attr_reader :if_operator_next
            attr_reader :statement_indent_if_operator_next

            def initialize(parse_result, scanner)
                @parse_result = parse_result
                @scanner = scanner
                @prev_is_space = :newline
                @current_indent = nil
                @prefer_operand_next = true
                @if_operand_next = nil
                @if_operator_next = nil
                @statement_indent_if_operand_next = nil
                @statement_indent_if_operand_next = nil
            end

            def output
                scanner.output
            end

            def scan_next
                token_start = scanner.index
                if token_type = scanner.next
                    token_end = scanner.index
                    next_is_space = scanner.next_is_space?
                    advance(token_start, token_type, token_end, next_is_space)
                    self
                end
            end

            def advance(token_start, token_type, token_end, next_is_space)
                space = surrounding_space(next_is_space) unless token_start == token_end

                output.debug "- token #{token_type.name} (#{token_start}-#{token_end}), prefer #{prefer_operand_next? ? "operand" : "operator"}, indent #{current_indent}#{space ? ", #{space} space" : ""}"

                # Read and resolve the next term
                action, prefer_operand_next, if_operand, if_operator =
                    token_type.next_state(prefer_operand_next?, space)
                output.debug "  action: #{action}, next: #{prefer_operand_next}, if_operand: #{if_operand.fixity}, if_operator: #{if_operator.fixity}"
                set_statement_indent(if_operand, if_operator)
                set_line_data(token_start, token_type, token_end, next_is_space)
                resolve_left(action, if_operand.left_is_operand?) if action
                resolve_term(token_start, token_end, if_operand, if_operator)
                set_statement_boundary(if_operand, if_operator)

                # Set the next preference
                @prefer_operand_next = prefer_operand_next
                @prev_is_space = token_type.space unless token_start == token_end

                if if_operand.grammar && if_operand.grammar != scanner.grammar
                    @scanner = if_operand.grammar.scanner(scanner.stream)
                end

                # Auto-insert empty/apply if needed
                if (if_operand.significant? && if_operand.right_is_operand?) ||
                   (if_operator.significant? && !if_operator.right_is_operand?)
                    advance(token_end, scanner.grammar.insert, token_end, next_is_space)
                end
            end

            def prefer_operand_next?
                prefer_operand_next
            end

            def possibilities_to_s(indent)
                root = syntax_tree.root
                result = "#{indent}resolved so far: #{root ? root.expression_to_s : ""}"
                result << "\n#{indent} if next is operand : #{if_operand_next.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}" if if_operand_next.any?
                result << "\n#{indent} if next is operator: #{if_operator_next.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}" if if_operator_next.any?
                result
            end

            private

            def set_line_data(token_start, token_type, token_end, next_is_space)
                if prev_is_space == :newline
                    if token_type.space == :whitespace
                        parse_result.line_data.append_line(token_start, token_end, next_is_space)
                    else
                        parse_result.line_data.append_line(token_start, token_start, token_type == :newline)
                    end
                    @current_indent = parse_result.line_data.current_indent
                end
            end

            def set_statement_indent(if_operand, if_operator)
                @statement_indent_if_operand_next ||= current_indent if if_operand.significant?
                @statement_indent_if_operator_next ||= current_indent if if_operator.significant?
            end

            def set_statement_boundary(if_operand, if_operator)
                if if_operand.statement_boundary
                    @statement_indent_if_operand_next = nil
                end
                if if_operator.statement_boundary
                    @statement_indent_if_operator_next = nil
                end
            end

            def surrounding_space(next_is_space)
                # Return trailing / leading if we're surrounded on one side by space
                if prev_is_space && !next_is_space
                    :leading
                elsif !prev_is_space && next_is_space
                    :trailing
                end
            end

            #
            # Deal with any ambiguities
            #
            def resolve_left(action, next_is_operand)
                # Handle the left side according to what we've been told.
                case action
                when :resolve
                    terms = next_is_operand ? if_operand_next : if_operator_next
                    if terms
                        output.debug "  left sides are both #{next_is_operand ? "operand" : "operator"}, resolving to #{terms.map { |s,e,i,type| "#{type}(#{type.fixity})" }.join(" ")}"
                        terms.each do |token_start, token_end, statement_indent, type|
                            parse_result.append_term(token_start, token_end, statement_indent, type)
                        end
                    end
                    @if_operand_next = nil
                    @if_operator_next = nil
                when :swap
                    @if_operand_next, @if_operator_next = if_operator_next, if_operand_next
                    @statement_indent_if_operand_next, @statement_indent_if_operator_next = statement_indent_if_operator_next, statement_indent_if_operand_next
                when nil
                else
                    raise "Unknown action #{action}"
                end
            end

            def resolve_term(token_start, token_end, if_operand, if_operator)
                # Append the actual term if unambiguous.
                if if_operand == if_operator
                    statement_indent = if_operand.left_is_operand? ? statement_indent_if_operand_next : statement_indent_if_operator_next
                    return parse_result.append_term(token_start, token_end, statement_indent, if_operand)
                end

                @if_operand_next ||= []
                @if_operand_next << [ token_start, token_end, statement_indent_if_operand_next, if_operand ]

                @if_operator_next ||= []
                @if_operator_next << [ token_start, token_end, statement_indent_if_operator_next, if_operator ]
            end
        end
    end
end
