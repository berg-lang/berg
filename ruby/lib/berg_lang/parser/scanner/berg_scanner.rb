require_relative "berg_number_scanner"
require "forwardable"

module BergLang
    module Parser
        class Scanner
            class BergScanner
                attr_reader :context
                attr_reader :grammar
                attr_reader :prev_token

                def initialize(context, grammar)
                    super(context)
                    @grammar = grammar
                end

                include BergNumberScanner
                include Berg

                def open
                end

                def close
                    stream.release(stream.index)
                end

                def scan_expression
                    start_scan!
                    if scan_comment || scan_operator || scan_number || scan_identifier
                        skip_space
                        buffer
                    end
                end

                def scan_space
                    skip_space
                    symbol_string
                end

                def skip_space
                    start_symbol!
                    consume_all SPACE_CHARACTERS
                end

                def skip_newline
                    start_symbol!
                    consume_if NEWLINE_CHARACTERS
                end

                private

                def scan_comment
                    # Comment
                    if consume_if '#'
                        consume_until NEWLINE_CHARACTERS
                        buffer << token_with_string(single_line_comment)
                    end
                end

                def scan_operator
                    # + - ! ( ) ......
                    if operator = consume_if operators
                        pick_operator(operator)
                    end
                end

                #
                # Choose the preferred variant of the operator, if there is more than one.
                #
                def pick_operator(operator)
                    # Some operators are guaranteed to be unambiguous forever. These are
                    # exempt from the space rules. This mainly applies to (), {} and [].
                    if operator.forever_unambiguous?
                        buffer << operator.single

                    elsif leading_space? == trailing_space?
                        if operator.postfix && !leading_space?
                            # If there is no space on either side, and postfix is possible,
                            # see if there is a run of postfix leading to a space.
                            while next_operator = peek_if(postfix_operators)
                                buffer << token(operator.postfix)
                                start_symbol!
                                consume_if(postfix_operators)
                            end

                            # If the last one has trailing space, or is ) } or ], success!
                            if trailing_space? || operator.forever_unambiguous?
                                buffer << token(operator.postfix)

                            # If we have no more postfix but the operator also does infix,
                            # use that.
                            elsif buffer.size == first_postfix
                                buffer << token(operator.infix)

                            # Otherwise, mark the last postfix as an error.
                            else
                                buffer << error_token(error_prefix_or_postfix_with_equal_space)
                            end

                        # If there is no space on either side, and infix is possible, pick it.
                        elsif operator.infix
                            buffer << token(operator.infix)

                        # Otherwise, it must be prefix.
                        else
                            buffer << error_token(error_prefix_or_postfix_with_equal_space)
                        end

                    # If there is trailing space only, pick prefix.
                    elsif leading_space?
                        if operator.prefix
                            buffer << token(operator.prefix)
                            # If there are more prefixes, pick them too.
                            symbol_start!
                            while operator = consume_if(prefix_operators)
                                buffer << token(operator.prefix)
                                symbol_start!
                            end
                        else
                            error_infix_or_postfix_with_leading_space!
                        end

                    # If there is trailing space only, pick postfix.
                    else
                        if operator.postfix
                            buffer << token(operator.postfix)
                        else
                            error_infix_or_prefix_with_leading_space!
                        end
                    end
                end

                extend Forwardable

                attr_reader :state

                def start_symbol!
                    @symbol_start = stream.index
                    stream.release(stream.index - 1)
                end

                (BergGrammar.instance_methods - Object.instance_methods).each do |method|
                    def_delegators :grammar, method
                end

                def_delegators :stream, :consume_if, :consume_unless, :consume_all, :consume_until
                def_delegators :stream, :peek_if, :peek_unless

                def token_string(start = nil)
                    if start.nil?
                        return if symbol_start == stream.index
                        start = symbol_start
                    end
                    stream.substr(start...stream.index)
                    start_symbol!
                    stream.release(stream.index - 1)
                end

                def output_error(error_type, operation_type: :expression, start: nil)
                    error = Error.new(error_type, symbol_range(start))
                    context.syntax_tree.errors << error
                    output error.send(operation_type), start: start
                end

                def consume_newline
                    # Record the line start data
                    if result = consume_if(newline_strings)
                        syntax_tree.source_data.line_start_codepoint_indices << stream.index
                    end
                    result
                end

                IDENTIFIER_START = [ "A".."Z", "a".."z", "_" ]
                IDENTIFIER_CHARACTERS = [ "A".."Z", "a".."z", "0".."9", "_" ]
                SPACE_CHARACTERS = [ " ", "\t" ]
                NEWLINE_CHARACTERS = [ "\r\n", "\n", "\r" ]

                private

                def output(symbol, start)
                    syntax_tree_builder.output symbol
                    true
                end

                def output_error(symbol, start)
                    syntax_tree_builder.output 
                end

                def output!(symbol)
                    syntax_tree_builder.output!(symbol)
                    true
                end
            end
        end
    end
end