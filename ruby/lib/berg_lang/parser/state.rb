module BergLang
    class Parser
        class State
            attr_reader :syntax_tree
            attr_reader :insert_if_non_preferred

            def initialize(source, prefer_operand_next:, insert_if_non_preferred:)
                @syntax_tree = SyntaxTree.new(source)
                set(prefer_operand_next, insert_if_non_preferred)
            end

            def prefer_operand_next?
                @prefer_operand_next
            end

            #
            # Get the terms that need to be inserted to resolve this.
            #
            def resolve(next_is_operand)
                preferred = next_is_operand == prefer_operand_next?
                insert_type = insert_if_non_preferred if !preferred
                terms = next_is_operand ? if_next_is_operand : if_next_is_operator
                [ insert_type, terms ]
            end

            # Set unresolved = right sides differ. infix/postfix, prefix/expression, infix/filler, filler/expression.
            def set_unresolved(term_start, term_end, if_operand, if_operator)
                if if_operand.filler? || if_operator.filler?
                    prefer_operand = if_operator.filler?
                else
                    prefer_operand = if_operand.left_is_operand? == if_operand.right_is_operand?
                end
                set(prefer_operand, nil)
                if_next_is_operand << [ term_start, term_end, if_operand ] unless if_operand.filler?
                if_next_is_operator << [ term_start, term_end, if_operator ] unless if_operator.filler?
            end

            # Unresolved = left sides and right sides both differ. infix/expression, prefix/postfix, prefix/filler, filler/postfix. infix/expression swaps our current preference, others leave it alone.
            def append_unresolved(term_start, term_end, if_operand, if_operator)
                if if_operand.left_is_operand? == if_operand.right_is_operand? && !if_operand.filler?
                    @if_next_is_operand, @if_next_is_operator = if_next_is_operator, if_next_is_operand
                    @prefer_operand_next = !prefer_operand_next
                end
                if_next_is_operand << [ term_start, term_end, if_operand ] unless if_operand.filler?
                if_next_is_operator << [ term_start, term_end, if_operator ] unless if_operator.filler?
            end

            def set(prefer_operand_next, insert_if_non_preferred)
                @prefer_operand_next = prefer_operand_next
                @insert_if_non_preferred = insert_if_non_preferred
                @if_next_is_operand = []
                @if_next_is_operator = []
            end

            def possibilities_to_s(indent)
                root = syntax_tree.root
                result = "#{indent}resolved so far: #{root ? root.expression_to_s : ""}"
                if_operand = resolved_terms(0, true)
                if_operator = resolved_terms(0, false)
                result << "\n#{indent} if next is operand : #{if_operand.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}" if if_operand.any?
                result << "\n#{indent} if next is operator: #{if_operator.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}" if if_operator.any?
                result
            end

            private

            attr_reader :prefer_operand_next
            attr_reader :if_next_is_operand
            attr_reader :if_next_is_operator
        end
    end
end
