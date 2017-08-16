module BergLang
    module Parser
        class UnresolvedSymbol < Struct[:noun, :verb, :start_index, :string, :leading_indent]
        end
    end
end