require_relative "../token"

module BergLang
    class Parser
        class Operator
            include Token

            attr_reader :operator_definitions

            def initialize(match, operator_definitions)
                super(match)
                @operator_definitions = operator_definitions
            end

            def to_s
                operator_definitions.values.first.to_s
            end

            def key
                operator_definitions.values.first.key
            end

            def prefix
                operator_definitions[:prefix] || operator_definitions[:start_delimiter]
            end

            def infix
                operator_definitions[:infix]
            end

            def postfix
                operator_definitions[:postfix] || operator_definitions[:end_delimiter]
            end

            def start_delimiter
                operator_definitions[:start_delimiter]
            end

            def end_delimiter
                operator_definitions[:end_delimiter]
            end
        end
    end
end