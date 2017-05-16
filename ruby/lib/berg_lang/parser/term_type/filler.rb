require_relative "../term_type"

module BergLang
    class Parser
        class TermType
            class Filler < TermType
                def initialize(name, newline: false, whitespace: newline)
                    super(name)
                    @newline = newline
                    @whitespace = whitespace
                end

                def filler?
                    true
                end
                def newline?
                    @newline
                end
                def whitespace?
                    @whitespace
                end
            end
        end
    end
end
