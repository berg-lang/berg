require_relative "scanner"
require_relative "syntax_tree"
require_relative "term_type/ambiguous"
require_relative "syntax_errors"

module BergLang
    class Parser
        #
        # Reads and disambiguates operator and expression tokens from the source.
        #
        # Handles operator disambiguation, indent/undent, newline as operator, sticky prefix/postfix, apply (a b
        # is a function call), and empty (e.g. a: 1, b:, c: has empty for b and c).
        #
        # Does not handle precedence and parent hookup.
        #
        class Resolver
            attr_reader :parser
            attr_reader :scanner
            attr_reader :syntax_tree

            def initialize(parser)
                @parser = parser
                @scanner = Scanner.new(parser)
                @syntax_tree = parser.syntax_tree
                @buried_open_indents = []
                @open_indent_size = -1
                syntax_tree.append_line(scanner.index, peek_indent)
            end

            #
            # Parse the next term.
            #
            # Won't stop until it hits something unambiguous.
            #
            # Inserts empty, apply, or newline if the resolved term does not match
            # what is needed (operator vs. expression).
            #
            # Handles indent and undent as well (when it does this, it does not
            # consume the newline).
            #
            def parse_next(left_prefers_operand, expect_indent_block, left_will_insert)
                leading_space = false
                result = nil

                while scanner.peek && scanner.peek.filler?
                    leading_space ||= true
                    if scanner.next.newline?
                        leading_space ||= :newline
                        term = handle_indent(expect_indent_block)
                        if term
                            # If we did cause an indent or undent, we need to continue parsing in order to inform the
                            # next term that it has a leading newline.
                            expect_indent_block = nil
                            result ||= term
                        end
                    end
                end

                term = parse_term(left_prefers_operand, left_will_insert, leading_space)
                result || term
            end
            
            private

            include SyntaxErrors

            NONE = 0
            EMPTY = 1
            APPLY = 2
            NEWLINE = 3
            NO_NEED_TO_INSERT = 4

            attr_reader :buried_open_indents
            attr_reader :open_indent_size

            #
            # Reads and resolves a term, adding it to the syntax tree.
            #
            # Will insert apply or empty in front of the term if it cannot satisfy
            # `left_prefers_operand`.
            #
            def parse_term(left_prefers_operand, left_will_insert, leading_space)
                term_start = scanner.index
                type = scanner.next
                term_end = scanner.index
                return nil unless type

                output.debug "#{type}: Resolving (prefer #{left_prefers_operand ? "operand" : "operator"}):"
                output.indented do

                    trailing_space = scanner.peek && scanner.peek.filler?

                    # Handle empty/apply if no one else has.
                    will_insert = left_prefers_operand ? EMPTY : APPLY
                    will_insert = NEWLINE if will_insert == APPLY && leading_space == :newline
                    will_insert = nil if left_will_insert >= will_insert

                    # Append the term and resolve its fixity.
                    term = syntax_tree.append(term_start, term_end)
                    term.type = choose_fixity(type, left_prefers_operand, will_insert || left_will_insert, leading_space, trailing_space)

                    # Insert the empty/apply if we need to.
                    if will_insert && left_prefers_operand != term.type.left_is_operand?
                        output.debug("Inserting #{insert_type(will_insert).name} because #{term.type.name} is #{term.type.fixity} (prefer #{left_prefers_operand})")
                        term = term.insert(term_start, term_start, insert_type(will_insert))
                    end
                    term
                end
            end

            #
            # Decide a term's fixity (expression, infix, postfix, prefix).
            #
            # We choose according to these rules:
            # - If we are sticky prefix or sticky postfix, we rule out infix entirely.
            # - If there is only one choice, we pick that.
            # - Otherwise, we resolve the next token and try to satisfy *both* sides first, or the right side if not.
            #
            def choose_fixity(type, left_prefers_operand, left_will_insert, leading_space, trailing_space)
                if !type.is_a?(TermType::Ambiguous)
                    output.debug("Resolving #{type.name} as #{type.fixity} because it is unambiguous.")
                    return type
                end

                # Remove infix as a possibility if we are sticky prefix/postfix
                if type.prefix? && type.infix? && leading_space && !trailing_space
                    output.debug("Sticky prefix: eliminating infix as a possibility for #{type.name}.")
                elsif type.postfix? && type.infix? && !leading_space && trailing_space
                    output.debug("Sticky postfix: eliminating infix as a possibility for #{type.name}.")
                else
                    infix = type.infix
                end

                # We prefer whatever will satisfy both sides. If we can't, we satisfy the right side.
                preferred = left_prefers_operand ? type.expression || type.postfix : infix   || type.prefix
                secondary = left_prefers_operand ? type.prefix     || infix        : type.postfix || type.expression

                if preferred && secondary
                    # If the left sides are the same, we can pick whatever; prefer expression/infix over
                    # postfix/prefix, and make sure the next term doesn't try to insert anything.
                    if !preferred.left == !secondary.left
                        preferred, secondary = secondary, preferred if preferred.prefix? || preferred.postfix?
                        left_will_insert = NO_NEED_TO_INSERT
                    end

                    # Parse the next term and decide whether we are ambiguous from that.
                    output.debug("Reading next token to decide whether #{type.name} should be #{preferred.fixity} or #{secondary.fixity} ...")
                    # NOTE: opens_indent_block? is guaranteed to be false if expr/post, see TermType::Ambiguous.validate
                    term = parse_next(preferred.right, nil, left_will_insert)
                    preferred, secondary = secondary, nil unless preferred.right_is_operand? == !!term.type.left
                    output.debug("Resolving #{type.name} as #{preferred.fixity} because right side is #{term.type.fixity}")
                else
                    output.debug("Resolving #{type.name} as #{(preferred || secondary).fixity} because it's the only thing that fits the left side.")
                end

                # If only one thing is possible, return it!
                preferred || secondary
            end

            #
            # Deal with indent and undent.
            #
            def handle_indent(expect_indent_block)
                indent = read_indent
                syntax_tree.add_line(token_end, indent)

                # Handle indent/undent if the token after the indent is visible (comment or term)
                if !scanner.peek.whitespace?
                    if expect_indent_block && indent > expect_indent_block
                        # Handle indent
                        buried_open_indents << open_indent_size if open_indent_size >= 0
                        @open_indent_size = expect_indent_block
                        return syntax_tree.append(token_start, token_start, parser.terms.indent)
                    else
                        result = nil
                        while indent <= open_indent.size
                            term = syntax_tree.append(token_start, token_start, parser.terms.indent)
                            result ||= term
                            @open_indent_size = buried_open_indents.pop || -1
                        end
                        return term
                    end
                end
            end

            #
            # Read the next whitespace token (if any) to determine indent size.
            #
            def read_indent
                result = peek_indent
                scanner.next if result > 0
                result
            end

            #
            # Peek at the next whitespace token (if any) to determine indent size.
            #
            def peek_indent
                return 0 unless scanner.peek && scanner.peek.whitespace?
                scanner.peek_index - scanner.index
            end

            #
            # Get the type of token that needs to be inserted for a given will_insert value.
            #
            def insert_type(will_insert)
                case will_insert
                when EMPTY
                    parser.terms.empty
                when APPLY
                    parser.terms.apply
                when NEWLINE
                    parser.terms.newline_operator
                end
            end
        end
    end
end
