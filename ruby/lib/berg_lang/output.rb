module BergLang
    class Output
        attr_reader :stream
        attr_reader :default_indent
        attr_reader :indent

        def initialize(stream, default_indent: "  ")
            @stream = stream
            @default_indent = default_indent
            @indent = ""
        end

        def indented(indent = default_indent, &block)
            old_indent = @indent
            begin
                @indent = "#{old_indent}#{indent}"
                block.call
            ensure
                @indent = old_indent
            end
        end

        def debug(string)
            puts "#{indent}#{string}"
        end

        def puts(string)
            stream.puts("#{indent}#{string}")
        end
    end
end
