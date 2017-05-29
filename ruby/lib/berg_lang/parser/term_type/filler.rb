require_relative "../term_type"

module BergLang
    class Parser
        class TermType
            class Filler < TermType
                def initialize(name, whitespace:)
                    super(name)
                    @whitespace = whitespace
                end

                def fixity
                    :filler
                end
                def filler?
                    true
                end
                def filler
                    self
                end
                def whitespace?
                    @whitespace
                end
                def left
                    nil
                end
                def right
                    nil
                end
            end
        end
    end
end
