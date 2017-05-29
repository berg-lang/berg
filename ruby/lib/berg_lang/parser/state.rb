module BergLang
    class Parser
        class State
            attr_reader :syntax_tree

            def initialize(source, prefer_next_to_be_operand:, insert_if_need_operand:, insert_if_need_operator:)
                @syntax_tree = SyntaxTree.new(source)
                @insert_if_need_operand = insert_if_need_operand
                @insert_if_need_operator = insert_if_need_operator
                reset(prefer_next_to_be_operand: prefer_next_to_be_operand)
            end

            def prefer_next_to_be_operand?
                @prefer_next_to_be_operand
            end

            def resolved_terms(term_start, type)
                return enum_for(:resolved) unless block_given?
                preferred = type.left_is_operand? == prefer_next_to_be_operand
                terms = type.left_is_operand? ? if_next_is_operand : if_next_is_operator
                if preferred && !prefer_next_to_be_operand.nil?
                    term_start, term_end, type = terms.first if terms.any?
                    insert_type = type.left_is_operand? ? insert_if_need_operator : insert_if_need_operand
                    yield [ term_start, term_start, insert_type ]
                end
                terms.each { |term_start, term_end, type| yield [ term_start, term_end, type ] }
            end

            def reset(prefer_next_to_be_operand:)
                @prefer_next_to_be_operand = prefer_next_to_be_operand
                @if_next_is_operand = []
                @if_next_is_operator = []
            end

            def swap
                @if_next_is_operand, @if_next_is_operator = if_next_is_operator, if_next_is_operand
                @prefer_next_to_be_operand = !prefer_next_to_be_operand? unless prefer_next_to_be_operand?.nil?
            end

            def append(if_next_is_operand, if_next_is_operator)
                @if_next_is_operand << [term_start, term_end, if_next_is_operand] unless if_next_is_operand.filler?
                @if_next_is_operator << [term_start, term_end, if_next_is_operator] unless if_next_is_operator.filler?
            end

            private

            attr_reader :prefer_next_to_be_operand
            attr_reader :if_next_is_operand
            attr_reader :if_next_is_operator
            attr_reader :insert_if_need_operand
            attr_reader :insert_if_need_operator
        end
    end
end
