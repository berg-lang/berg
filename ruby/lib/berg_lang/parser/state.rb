module BergLang
    class Parser
        class State
            attr_reader :syntax_tree
            attr_accessor :prefer_operand_next
            attr_reader :if_operand_next
            attr_reader :if_operator_next

            def initialize(source, prefer_operand_next)
                @syntax_tree = SyntaxTree.new(source)
                @prefer_operand_next = prefer_operand_next
                @if_operand_next = []
                @if_operator_next = []
            end

            def prefer_operand_next?
                @prefer_operand_next
            end

            #
            # Get the terms that need to be inserted to resolve this.
            #
            def resolve(next_is_operand)
                terms = next_is_operand ? if_operand_next : if_operator_next
                @if_operand_next = []
                @if_operator_next = []
                terms
            end

            #
            # Swap if_operand_next and if_operator_next
            #
            def swap_unresolved
                @if_operand_next, @if_operator_next = if_operator_next, if_operand_next
            end

            def possibilities_to_s(indent)
                root = syntax_tree.root
                result = "#{indent}resolved so far: #{root ? root.expression_to_s : ""}"
                result << "\n#{indent} if next is operand : #{if_operand_next.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}" if if_operand_next.any?
                result << "\n#{indent} if next is operator: #{if_operator_next.map { |s,e,type| "#{type}(#{type.fixity})" }.join(" ")}" if if_operator_next.any?
                result
            end
        end
    end
end
