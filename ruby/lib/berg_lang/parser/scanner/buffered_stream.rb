module BergLang
    module Parser
        #
        # Interface representing a source stream
        #
        module BufferedStream
            #
            # The parser context.
            #
            attr_reader :context

            #
            # The current character index.
            #
            attr_reader :index

            #
            # We can reset up to this point. `release` sets this.
            #
            attr_reader :keep_index

            #
            # Create a new SourceStream with the given source.
            #
            def initialize(context)
                @context = context
                @index = 0
                @keep_index = 0
            end

            #
            # The source of this stream.
            #
            def source
                context.source
            end

            #
            # Whether this SourceStream has any more characters or not.
            #
            def eof?
                peek.nil?
            end

            #
            # Peek at the next character without advancing.
            #
            # @return [String,nil] The peeked character, or nil if EOF.
            #
            def peek
                substr(index..index)
            end

            #
            # Reset to the given index.
            #
            def rewind(index)
                raise ArgumentError, "#{index} is less than keep_index #{keep_index}" if index < keep_index
                raise ArgumentError, "#{index} is bigger than current index #{self.index}" if index > self.index
                @index = index
            end

            #
            # Get the substring specified by the range.
            #
            def substr(range)
                raise NotImplementedError, "#{self.class}.substr"
            end

            #
            # Release everything up to (but not including) the given index.
            #
            # @param amount [Integer] The index into the file we no longer release.
            #
            def release(index)
                raise ArgumentError, "#{index} is less than keep_index #{"
                raise ArgumentError, "#{index} is less than keep_index #{keep_index}" if index < keep_index
                raise ArgumentError, "Cannot release backwards to #{index} because it is beyond current index #{self.index}" if index > self.index
                @index = index
            end


            #
            # Consume one character, returning it and advancing.
            #
            # @return `true` if we could advance, or `nil` if EOF.
            #
            def consume
                result = peek
                advance if result
                result
            end

            #
            # Advance ahead the specified number of characters.
            #
            # @param amount [Integer] The number of characters to advance.
            # @return [Integer,nil] The number of characters advanced. Will be
            # less than `amount` if there are not enough characters before EOF.
            #
            def advance(amount=1)
                raise NotImplementedError, "#{self.class}.advance"
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
            #
            # @return [Object,true,false,nil] true (or the Hash value of the match) if
            #   anything was found, false if nothing found, and nil if EOF.
            #
            # @raise ArgumentError if no matchers are given or if a range matcher has
            #    different sizes on the min and max of the range.
            #
            def consume_if(*matchers)
                size, result = peek_match(*matchers)
                if size
                    advance(size)
                    return result || true
                else
                    return size
                end
            end

            #
            # Consume input as long as it matches the given matchers.
            #
            # @param matchers [String,Range] matchers or characters to match. Matches
            #   prioritized in order, so longest should be first.
            #
            # @return [true,false,nil] true if anything was found, false if nothing
            #   found, and nil if EOF.
            #
            def consume_all(*matchers)
                result = consume_if(*matchers)
                return result if !result
                while consume_if(*matchers); end
                return true
            end

            #
            # Consume the next character, *unless* the given string is found.
            #
            # @param matchers [String,Range] matchers or characters to avoid. Matches
            #   prioritized in order, so longest should be first if you are doing a
            #   max-munch.
            # @param append_to [String,nil] A string to append the data to, or nil to
            #    not save the data at all.
            #
            # @return [true,false,nil] true if anything was found, false if nothing
            #   found, and nil if EOF.
            #
            def consume_unless(*matchers)
                size, result = peek_match(*matchers)
                return false if size
                consume
            end

            #
            # Consume input as long as it does *not* match the given matchers.
            #
            # @param matchers [String,Range] matchers or characters to match. Matches
            #   prioritized in order, so longest should be first.
            #
            # @return [true,false,nil] true if anything was found, false if nothing
            #   found, and nil if EOF.
            #
            def consume_until(*matchers)
                result = consume_unless(*matchers)
                return result if !result
                while consume_unless(*matchers); end
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
