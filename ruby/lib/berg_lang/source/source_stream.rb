module BergLang
    module Parser
        #
        # Interface representing a source stream
        #
        module SourceStream
            #
            # The Source of this stream
            #
            attr_reader :source

            #
            # The codepoint index of the buffer
            #
            attr_reader :codepoint_index

            #
            # The codepoint buffer (string)
            #
            attr_reader :buffer

            #
            # Create a new SourceStream with the given source.
            #
            def initialize(source)
                @source = source
                @codepoint_index = 0
            end

            #
            # Whether this SourceStream has any more codepoints or not.
            #
            def eof?
                peek.nil?
            end

            #
            # Get a future codepoint, without consuming it.
            #
            # @param lookahead_size [Integer] The future codepoint number of codepoints to peek at.
            #
            # @return [String,nil] The peeked string, or nil if EOF.
            #
            def peek(lookahead=1)
                raise NotImplementedError, "#{self.class}.consume"
            end

            #
            # Consume one or more codepoints.
            #
            # @param lookahead_size [Integer] The number of codepoints to consume.
            # @param append_to [String,nil] A string to append the data to, or nil to
            #    not save the data at all.
            #
            # @return [true,nil] true if any codepoints were consumed, or nil if there
            #    are not enough codepoints until EOF.
            #
            def consume(lookahead_size=1, append_to: nil)
                raise NotImplementedError, "#{self.class}.consume"
            end

            #
            # Consume the input for the first matching matcher.
            #
            # @param [Hash,String,Range] matchers The matchers to test. If it is a hash,
            #    each key will be tested against the string and if it matches, the value
            #    will be returned. If a string or a range, it is tested and true is
            #    returned. Ranges MUST have the same size string on the left and right.
            #    Matches are prioritized in the order passed, so longest should be
            #    first if you are doing a max munch thing.
            # @param append_to [String,nil] A string to append the data to, or nil to
            #    not save the data at all.
            #
            # @return [Object,true,false,nil] true (or the Hash value of the match) if
            #   anything was found, false if nothing found, and nil if EOF.
            #
            # @raise ArgumentError if no matchers are given or if a range matcher has
            #    different sizes on the min and max of the range.
            #
            def consume_if(*matchers, append_to: nil)
                size, result = peek_match(*matchers)
                if size
                    consume(size, append_to: append_to)
                    return result || true
                else
                    return size
                end
            end

            #
            # Consume input as long as it matches the given matchers.
            #
            # @param matchers [String,Range] matchers or codepoints to match. Matches
            #   prioritized in order, so longest should be first.
            # @param append_to [String,nil] A string to append the data to, or nil to
            #    not save the data at all.
            #
            # @return [true,false,nil] true if anything was found, false if nothing
            #   found, and nil if EOF.
            #
            def consume_all(*matchers, append_to: nil)
                result = consume_if(*matchers, append_to: append_to)
                return result if !result
                while consume_if(*matchers, append_to: append_to)
                end
                return true
            end

            #
            # Consume the next codepoint, *unless* the given string is found.
            #
            # @param matchers [String,Range] matchers or codepoints to avoid. Matches
            #   prioritized in order, so longest should be first if you are doing a
            #   max-munch.
            # @param append_to [String,nil] A string to append the data to, or nil to
            #    not save the data at all.
            #
            # @return [true,false,nil] true if anything was found, false if nothing
            #   found, and nil if EOF.
            #
            def consume_unless(*matchers, append_to: nil)
                size, result = peek_match(*matchers)
                if size
                    return false
                else
                    return consume(append_to: append_to)
                end
            end

            #
            # Consume input as long as it does *not* match the given matchers.
            #
            # @param matchers [String,Range] matchers or codepoints to match. Matches
            #   prioritized in order, so longest should be first.
            #
            # @return [true,false,nil] true if anything was found, false if nothing
            #   found, and nil if EOF.
            #
            def consume_until(*matchers)
                result = consume_unless(*matchers, append_to: append_to)
                return result if !result
                while consume_unless(*matchers, append_to: append_to)
                end
                return true
            end

            #
            # Peek whether one or more of the given matchers matches, without consuming
            # any input.
            #
            # @param [Hash,String,Range] matchers The matchers to test. If it is a hash,
            #    each key will be tested against the string and if it matches, the value
            #    will be returned. If a string or a range, it is tested. Matches are
            #    prioritized in the order passed, so longest should be first if you are
            #    doing a max munch thing.
            #
            # @return [<[Integer,false,nil], [Object,nil]>] the size if anything
            #    was found, and the resulting object if it was in a Hash.
            #
            # @raise ArgumentError if no matchers are given, if any matchers are *not*
            #    Hash, String or Range, if any matcher Hash result is nil or false, or
            #    if a range matcher has different sizes on the min and max of the range.
            #
            def peek_match(*matchers)
                if matchers.empty?
                    raise ArgumentError, "No matchers passed to peek_match!"
                end

                return nil if eof?

                matchers.each do |matcher|
                    case matcher
                    when Hash
                        matcher.each do |matcher, result|
                            if result.nil? || result == false
                                raise ArgumentError, "Bad matcher hash result #{result.inspect}"
                            end
                            size, result = peek_match(matcher)
                            return [ size, result ] if size
                        end
                    when String
                        str = peek(matcher.size)
                        return str.size if str == matcher
                    when Range
                        if matcher.min.size != matcher.max.size
                            raise ArgumentError, "Range has different size on left and right"
                        end
                        str = peek(matcher.min.size)
                        return str.size if matcher.include?(str)
                    else
                        raise ArgumentError, "Unexpected type #{string.class} of #{string.inspect}"
                    end
                end

                return false
            end
        end
    end
end
