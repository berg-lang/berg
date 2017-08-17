module BergLang
    module Parser
        class Token < Struct[:token_type, :range, :string, :parent_index]
        end
    end
end