module BergLang
    #
    # Represents a Berg source.
    #
    module Source
        def name
            raise NotImplementedException, "#{self.class}.name"
        end
    end
end
