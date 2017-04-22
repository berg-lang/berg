require_relative "../token"

module BergLang
    class Parser
        class Operator
            include Token

            attr_reader :operator_definitions

            def initialize(source_range, operator_definitions)
                super(source_range)
                @operator_definitions = operator_definitions
            end

            def to_s
                operator_definitions.values.first.to_s
            end

            def key
                operator_definitions.values.first.key
            end

            def opens_indent_block?
                (prefix && prefix.opens_indent_block?) ||
                  (infix && infix.opens_indent_block?)
            end

            def prefix
                operator_definitions[:prefix] || operator_definitions[:open]
            end

            def infix
                operator_definitions[:infix]
            end

            def postfix
                operator_definitions[:postfix] || operator_definitions[:close]
            end

            def open
                operator_definitions[:open]
            end

            def close
                operator_definitions[:close]
            end
        end
    end
end