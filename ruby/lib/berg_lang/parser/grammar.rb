require_relative "token_type"
require_relative "scanner"
require "set"

module BergLang
    class Parser
        class Grammar
            attr_reader :output

            def initialize(output)
                @output = output
            end

            def scanner(stream, output=self.output)
                Scanner.new(self, stream, output)
            end

            def self.token_alias(*names)
                names.each do |name|
                    define_method(name) { tokens[name] }
                end
            end

            private

            def define_terms(*term_defs)
                #
                # Process the nice term string definitions
                #
                direction = nil
                tokens = term_defs.flat_map do |term_def|
                    if term_def.is_a?(String)
                        # String is like "* / + *"
                        term_def = term_def.split(/ /)

                        # If string starts with "right", like "right = += -=", use that as direction
                        if %w{left right}.include?(term_def.first)
                            direction ||= term_def.shift.to_sym
                        else
                            direction ||= :left
                        end

                        # Parse through looking for prefix, infix, etc. "++.postfix --.postfix"
                        term_def.map do |term_string|
                            if term_string =~ /^(.+)\.(.+)$/
                                define_term(string: $2, type: $1.to_sym)
                            else
                                define_term(string: term_string)
                            end
                        end
                    else
                        direction ||= term_def.delete(:direction)
                        indented_variant_name = term_def.delete(:indented_variant_name)
                        if indented_variant_name
                            indented_variant = define_term(**term_def, name: indented_variant_name, statement_boundary: :nest)
                        end
                        define_term(**term_def, indented_variant: indented_variant)
                    end
                end
                [ direction, tokens ]
            end

            def define_term(string: nil, name: string, token_name: name, type: :infix, opened_by: nil, closed_by: nil, declaration: nil, direction: nil, space: false, significant: !space, indented_variant: nil, statement_boundary: !!indented_variant)
                if [:infix, :postfix, :close ].include?(type)
                    left = { declaration: declaration, opened_by: opened_by }
                end
                if [:infix, :prefix, :open ].include?(type)
                    right = { closed_by: closed_by }
                end
                TermType.new(self, name, token_name: token_name, left: left, right: right, space: space, significant: significant, indented_variant: indented_variant, statement_boundary: statement_boundary)
            end

            def define_tokens(*groups)
                token_terms = {}
                term_groups = groups.map do |term_defs|
                    term_defs = Array(term_defs).flatten
                    direction, term_group = define_terms(*term_defs)
                    term_group.each do |term|
                        token_terms[term.token_name] ||= {}
                        if token_terms[term.token_name].has_key?(term.fixity)
                            raise "Token #{term.token_name} has multiple #{term.fixity} variants!"
                        end
                        token_terms[term.token_name][term.fixity] = term
                    end
                    [ direction, term_group ]
                end

                # Add opened_by and closed_by
                token_terms.each do |name, variants|
                    variants.each do |fixity, variant|
                        left = variant.left
                        right = variant.right
                        left.opened_by = token_terms[left.opened_by][:prefix] if left && left.opened_by
                        right.closed_by = token_terms[right.closed_by][:postfix] if right && right.closed_by
                    end
                end

                # Handle precedence.
                # The definition of precedence is that tokens at level n are unwilling to have tokens
                # at level n+1 as children. As you go up, you only have lower levels as children.
                # tokens at the same level are willing to have each other as left or right children,
                # depending on direction.
                all_terms = Set.new
                term_groups.each do |direction, term_group|
                    left_terms = all_terms
                    right_terms = all_terms
                    if direction == :left
                        left_terms = left_terms + term_group
                    else
                        right_terms = right_terms + term_group if direction != :left
                    end

                    term_group.each do |term|
                        term.left.accepts_operands += left_terms if term.left
                        term.right.accepts_operands += right_terms if term.right
                    end

                    all_terms += term_group
                end

                # Sort result by precedence.
                token_terms.map do |name, terms|
                    term = terms.values.first
                    [ name, TokenType.new(self, name, string: term.string, **terms) ]
                end.sort_by { |name, value| name.is_a?(String) ? -name.size : 0 }.to_h
            end
        end
    end
end