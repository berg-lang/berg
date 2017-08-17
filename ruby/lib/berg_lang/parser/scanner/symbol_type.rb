module BergLang
    module Parser
        class SymbolType
            attr_reader :name
            attr_reader :string
            attr_reader :expression
            attr_reader :binary
            attr_reader :prefix
            attr_reader :suffix

            def initialize(name, string)
                @name = name
                @string = string
            end

            def <<(token_type)
                add_token_type(token_type)
            end

            def add_token_type(token_type)
                operation_type = token_type.operation_type
                current_operation_type = instance_variable_get("@#{operation_type}")
                if current_operation_type
                    raise ArgumentError, "Cannot set #{operation_type} #{name} twice! Is already #{current_operation_type}, tried to set to #{token_type}"
                end

                if operation_type == :expression || operation_type == :suffix
                    if (binary && token_type.next_grammar != binary.next_grammar)
                        raise "#{operation_type} #{token_type}'s grammar #{token_type.next_grammar} conflicts with binary's grammar #{binary.next_grammar}. Cannot have two separator grammars next, as token parsing in Berg must be unambiguous."
                    end
                    if (prefix && token_type.next_grammar != prefix.next_grammar)
                        raise "#{operation_type} #{token_type}'s grammar #{token_type.next_grammar} conflicts with prefix's next grammar #{prefix.next_grammar}. Cannot have two separator grammars next, as token parsing in Berg must be unambiguous."
                    end
                else
                    if (expression && token_type.next_grammar != expression.next_grammar)
                        raise "#{operation_type} #{token_type}'s grammar #{token_type.next_grammar} conflicts with expression's next grammar #{expression.next_grammar}. Cannot have two separator grammars next, as token parsing in Berg must be unambiguous."
                    end
                    if (suffix && token_type.next_grammar != suffix.next_grammar)
                        raise "#{operation_type} #{token_type}'s grammar #{token_type.next_grammar} conflicts with suffix's next grammar #{suffix.next_grammar}. Cannot have two separator grammars next, as token parsing in Berg must be unambiguous."
                    end
                end

                instance_variable_set("@#{operation_type}", token_type)
            end

        end
    end
end
