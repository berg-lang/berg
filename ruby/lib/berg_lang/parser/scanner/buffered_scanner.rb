require "forwardable"
require_relative "../scanner"

module BergLang
    module Parser
        module BufferedScanner
            include Scanner

            attr_reader :context
            attr_reader :stream

            extend Forwardable

            def initialize(context, stream)
                @context = context
                @stream = stream
                @buffer = context.buffer
                @index = 0
            end

            #
            # Look at the next symbol in the stream without consuming it.
            #
            # @return [SymbolType] The next symbol in the stream, or `nil` if EOF.
            #
            def peek
                scan if index >= buffer.size
                buffer[index]
            end

            #
            # Read at the next symbol in the stream.
            #
            # @return [SymbolType] The next symbol in the stream, or `nil` if EOF.
            #
            def consume
                result = peek
                @index += 1
                result
            end

            #
            # Reset back to the last released point in the stream.
            #
            def reset
                @index = 1 unless index == 0
            end

            #
            # Release all symbols up to the current point.
            #
            def release
                @buffer = buffer[(index - 1)...-1] unless index == 0
            end

            #
            # Look at the previous symbol in the stream.
            #
            # @return [SymbolType] The previous symbol in the stream, or `nil` if start
            #    of file.
            #
            def previous
                buffer[index - 1] unless index == 0
            end

            private

            attr_reader :buffer
            attr_reader :index
        end
    end
end
