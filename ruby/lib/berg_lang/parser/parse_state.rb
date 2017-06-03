module BergLang
    class Parser
        class ParseState
            attr_reader :syntax_tree

            #
            # The scanner that will be used for the next token.
            #
            attr_accessor :scanner
            attr_accessor :prefer_operand_next
            attr_accessor :if_operand_next
            attr_accessor :if_operator_next
            attr_accessor :prev_is_space

            def initialize(source, scanner, prefer_operand_next: true, prev_is_space: false, if_operand_next: nil, if_operator_next: nil)
                @syntax_tree = SyntaxTree.new(source)
                @scanner = scanner
                @prefer_operand_next = prefer_operand_next
                @prev_is_space = prev_is_space
                @if_operand_next = if_operand_next
                @if_operator_next = if_operator_next
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
                space = leading_or_trailing_space(next_is_space)
                syntax_tree.append_line(token_end) if token_type == scanner.grammar.newline

                output.debug "- token #{token_type.name} (#{token_start}-#{token_end}), prefer #{prefer_operand_next? ? "operand" : "operator"}#{space ? ", #{space} space" : ""}"

                # Read and resolve the next term
                action, prefer_operand_next, if_operand, if_operator =
                    token_type.next_state(prefer_operand_next?, space)
                output.debug "  action: #{action}, next: #{prefer_operand_next}, if_operand: #{if_operand.fixity}, if_operator: #{if_operator.fixity}"
                resolve_left(action, if_operand.left_is_operand?) if action
                resolve_term(token_start, token_end, if_operand, if_operator)

                # Set the next preference
                @prefer_operand_next = prefer_operand_next
                @prev_is_space = token_type.space? unless token_start == token_end
                if if_operand.grammar && if_operand.grammar != scanner.grammar
                    @scanner = if_operand.grammar.scanner(scanner.stream)
                end

                # Auto-insert empty/apply if needed
                if if_operand.right_is_operand? || !if_operator.right_is_operand?
                    advance(token_end, scanner.grammar.insert, token_end, next_is_space)
                end
            end

            def prev_is_space?
                prev_is_space
            end

            def prev_is_space?
                prev_is_space
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

            def leading_or_trailing_space(next_is_space)
                if prev_is_space? != next_is_space
                    next_is_space ? :trailing : :leading
                end
            end

            def append_term(token_start, token_end, type)
                if !type.space?
                    output.debug "Appending #{type} (#{type.fixity})"
                    term = syntax_tree.append(token_start, token_end, type)
                    associate(term)
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
                        output.debug "  left sides are both #{next_is_operand ? "operand" : "operator"}, resolving to #{terms.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}"
                        terms.each do |token_start, token_end, type|
                            append_term(token_start, token_end, type)
                        end
                    end
                    @if_operand_next = nil
                    @if_operator_next = nil
                when :swap
                    @if_operand_next, @if_operator_next = if_operator_next, if_operand_next
                when nil
                else
                    raise "Unknown action #{action}"
                end
            end

            def resolve_term(token_start, token_end, if_operand, if_operator)
                # Append the actual term if unambiguous.
                if if_operand == if_operator
                    append_term(token_start, token_end, if_operand)
                else
                    if !if_operand.space?
                        @if_operand_next ||= []
                        @if_operand_next << [ token_start, token_end, if_operand ]
                    end
                    if !if_operator.space?
                        @if_operator_next ||= []
                        @if_operator_next << [ token_start, token_end, if_operator ]
                    end
                end
            end


            #
            # Associates an operator with the rest of the tree by setting its parent and
            # child correctly.
            #
            # @param [Term] The term in the tree to associate.
            #
            def associate(term)
                parent = term.previous_term
                return unless parent
                # If we have room for left children, pick the widest left child we can.
                type = term.type
                if type.left
                    while parent && type.left_accepts_operand?(parent.type)
                        left_operand, parent = parent, parent.parent
                    end
                    if !left_operand
                        raise internal_error(term, "#{term} (#{term.type.fixity} #{type}) cannot have left child #{parent} (#{parent.type.fixity} #{parent.type})!")
                    end
                    # If we are a close parentheses, and the chosen parent is our open parentheses (hopefully!),
                    # we make ourselves the parent of the open, and take the open's parent ourselves.
                    if type.left.opened_by
                        if parent && type.left.opened_by == parent.type
                            left_operand, parent = parent, parent.parent
                        else
                            raise unmatched_close(parent, term)
                        end
                    elsif parent && !parent.type.right_accepts_operand?(type)
                        raise internal_error(term, "#{term} cannot have parent #{parent}!")
                    end

                    left_operand.parent = term
                end
                term.parent = parent
            end
        end
    end
end
