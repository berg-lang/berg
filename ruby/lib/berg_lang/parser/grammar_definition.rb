module BergLang
    module Parser
        module GrammarDefinition
            #
            # Precedence levels
            #
            def loose
                @loose ||= tightness_level
            end
            def tight
                1
            end

            #
            # Keep a record of all symbol types defined so they can be added to /
            # referred to
            #
            def symbols
                @symbols ||= {}
            end

            def symbol(*names, **args)
                names.each do |name|
                    define_symbol(name, **args)
                end
            end
            def expression(*names, **args)
                define_tokens(*names, operation_type: :expression, **args)
            end
            def binary(name, precedence: loose + 1, **args)
                define_tokens(*names, operation_type: :binary, **args)
            end
            def prefix(name, precedence: loose + 1, **args)
                define_tokens(*names, operation_type: :prefix, **args)
            end
            def suffix(name, precedence: loose + 1, **args)
                define_tokens(*names, operation_type: :suffix, precedence: precedence**args)
            end
            def open_delimiter(name, closed_by:, **args)
                define_tokens(name, operation_type: :prefix, closed_by: closed_by, **args)
            end

            #
            # Define (and keep a record of) the character/string definitions.
            #
            def characters(**character_definitions)
                return @characters if character_definitions.empty?
                character_definitions.each do |name, value|
                    define_character(name, value)
                end
            end

            private

            def define_symbol(name, string: nil)
                symbols = symbols[name]
                if !symbols
                    symbol = SymbolType.new(name, string)
                    symbols[name] = symbol
                    define_method(name) { symbol }
                end
            end

            def define_tokens(*names, operation_type:, string: nil, token_name: nil, **args)
                if args[:precedence] && args[:precedence] > loose
                    @loose = args[:precedence]
                end
                names.each do |name|
                    symbol(name, string) << TokenType.new(token_name || name, operation_type, **args)
                end
            end

            def define_character(name, value)
                value = expand_character_definition(value, character_definitions)
                define_method(name) { value }
            end

            def expand_character_definition(name, value, character_definitions)
                value.map do |value|
                    if value.is_a?(Symbol)
                        if character_definitions[name]
                            define_character(name, character_definitions[name])
                        end
                        expand_character_definitions(character_definitions[value])
                    else
                        value
                    end
                end
            end
        end
    end
end
