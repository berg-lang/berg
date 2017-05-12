require_relative "scanner"
require_relative "token"

module BergLang
    class Parser
        #
        # Scans the file and chunks the result into *tokens* with start, end, 
        class Tokenizer
            attr_reader :parser
            attr_reader :scanner

            def initialize(parser)
                @parser = parser
                @scanner = Scanner.new(parser)
                buffer_first_token
            end

            #
            # Reads a token, consolidating leading and trailing whitespace.
            #
            # @return [Token] The next token.
            #
            def read_token
                return nil unless @trailing_token_type

                # Appropriate the last trailing token and space
                leading_space = @trailing_space
                leading_newline = @trailing_newline
                indent_start = @indent_start
                indent_end = @indent_end
                token_start = @trailing_token_start
                token_type = @trailing_token_type
                token_end = scanner.index

                # Read trailing space and trailing token
                @trailing_space = token_end
                @trailing_newline = nil
                while index = scanner.index && next_token = scanner.scan_token && next_token.whitespace?
                    if next_token.newline?
                        @indent_start = scanner.index
                        @indent_end = nil
                    end
                    @trailing_newline ||= index if next_token.newline?
                end
                @indent_end = index if @indent_start
                @trailing_token_start = index
                @trailing_token_type = next_token

                Token.new(leading_space, leading_newline, indent_start, indent_end, token_start, token_type, token_end, @trailing_newline, @trailing_token_start)
            end

            private

            #
            # Reads in the first token from the file.
            #
            def buffer_first_token
                @trailing_space = scanner.index
                @trailing_newline = nil
                @trailing_token_type = scanner.scan
                @trailing_token_start = scanner.index
                raise "Expected first token to be non-whitespace" if @trailing_token_type.whitespace?
            end
        end
    end
end


            # We buffer the current raw token and whitespace.
            attr_reader :tokenizer
            attr_reader :token
            attr_reader :leading_newline
            attr_reader :indent_start
            attr_reader :indent_end

            def advance(open_indent_start, open_indent_end)
                prev_token = self.token

                @leading_newline = nil
                leading_space = tokenizer.index
                begin
                    token_start = tokenizer.index
                    token_type = tokenizer.read_token
                    if token_type.whitespace
                        # If it's a newline, record that and be ready to record the indent.
                        if token_type == parser.token_types.newline
                            @leading_newline = token_start
                            @indent_start = token_end

                        # Comments can end indent. They are visible.
                        elsif token_type == parser.token_types.comment && leading_newline
                            if should_undent?(token_start, open_indent_start, open_indent_end)
                                next_token = Token.new(parser.token_type.undent, leading_space, token_start)
                            end
                        end
                    else
                        next_token = Token.new(token_type, leading_space, token_start)
                    end
                end until next_token

                # Record the whole indent.
                @indent_end = next_token.start if next_token.leading_newline
                @token = next_token
                prev_token
            end

            private

            def output
                parser.output
            end

            def should_undent?(indent_end, open_indent_start, open_indent_end)
                indent_size = indent_end - indent_start
                open_indent_size = open_indent_end - open_indent_start
                size = [indent_size, open_indent_size].max

                # Truncate both indents and make sure they match as far as tabs/spaces go
                open_indent = source.substring(open_indent_start, open_indent_start + size)
                indent = source.substring(indent_start, indent_start + size)
                if open_indent != indent
                    raise syntax_errors.unmatchable_indent(open_indent_start, open_indent_end, indent_start, indent_end)
                end

                # If we're properly indented, we won't find any further smaller indents. Exit early.
                if indent_size <= open_indent_size
                    output.debug("Undent: #{indent.inspect} followed by newline")
                    true
                else
                    false
                end
            end
        end
    end
end