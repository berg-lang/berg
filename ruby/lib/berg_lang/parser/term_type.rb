module BergLang
    class Parser
        class TermType
            attr_reader :name

            def initialize(name)
                @name = name
            end

            def to_s
                name
            end

            def filler?
                false
            end
            def term?
                !filler?
            end
            def whitespace?
                false
            end
            def newline?
                false
            end
            def infix?
                false
            end
            def infix
                nil
            end
            def expression?
                false
            end
            def expression
                nil
            end
            def prefix?
                false
            end
            def prefix
                nil
            end
            def postfix?
                false
            end
            def postfix
                nil
            end

            def +(term_type)
                raise "#{self} cannot be combined with another term" if variants.empty?
                raise "#{term_type} cannot be combined with another term" if term_type.variants.empty?
                raise "#{self} and #{term_type} are both expressions and cannot be combined" if term_type.expression? && expression?
                raise "#{self} and #{term_type} are both infix and cannot be combined" if term_type.infix? && infix?
                raise "#{self} and #{term_type} are both prefix and cannot be combined" if term_type.prefix? && prefix?
                raise "#{self} and #{term_type} are both postfix and cannot be combined" if term_type.postfix? && postfix?
                Ambiguous.new(infix: term_type.infix || infix, expression: term_type.expression || expression, prefix: term_type.prefix || prefix, postfix: term_type.postfix || postfix)
            end

            def variants
                [ expression, infix, prefix, postfix ].reject { |variant| variant.nil? }
            end
        end
    end
end
