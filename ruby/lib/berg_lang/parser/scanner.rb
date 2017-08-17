module BergLang
    module Parser
        module Scanner
            #
            # Read at least one symbol from the stream into the end of the buffer.
            #
            # Does nothing if at EOF.
            #
            def scan
                raise NotImplementedError.new("#{self.class}.scan")
            end

            #
            # Called before scanning starts.
            #
            def start
                raise NotImplementedError.new("#{self.class}.start")
            end

            #
            # Called when scanning stops.
            #
            def stop
                raise NotImplementedError.new("#{self.class}.start")
            end
        end
    end
end
