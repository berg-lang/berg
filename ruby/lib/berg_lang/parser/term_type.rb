require_relative "term_type/ambiguous"

module BergLang
    class Parser
        class TermType
            attr_reader :name

            def initialize(name)
                @name = name
            end

            def to_s
                name.is_a?(String) ? name : name.inspect
            end

            def fixity
                :error
            end

            def filler?
                false
            end
            def filler
                nil
            end
            def term?
                infix? || expression? || postfix? || prefix?
            end
            def whitespace?
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

            def term(left, right)
                if left
                    right ? expression : prefix
                else
                    right ? postfix : infix
                end
            end

            def +(term_type)
                raise "#{self} cannot be combined with another term" if !variants.any?
                raise "#{term_type} cannot be combined with another term" if !term_type.variants.any?
                raise "#{self} and #{term_type} are both expressions and cannot be combined" if term_type.expression? && expression?
                raise "#{self} and #{term_type} are both infix and cannot be combined" if term_type.infix? && infix?
                raise "#{self} and #{term_type} are both prefix and cannot be combined" if term_type.prefix? && prefix?
                raise "#{self} and #{term_type} are both postfix and cannot be combined" if term_type.postfix? && postfix?
                Ambiguous.new(infix: term_type.infix || infix, expression: term_type.expression || expression, prefix: term_type.prefix || prefix, postfix: term_type.postfix || postfix)
            end

            def variants
                return enum_for(:variants) if !block_given?
                yield expression if expression?
                yield infix if infix?
                yield prefix if postfix?
                yield postfix if postfix?
                yield filler if filler?
            end
        end
    end
end
