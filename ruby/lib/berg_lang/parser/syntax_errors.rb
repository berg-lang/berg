require_relative "syntax_error"

module BergLang
    class Parser
        class SyntaxErrors
            # The below methods *generate* the actual public syntax error methods
            private

            def self.syntax_error(name, error:, remedy:)
                define_method(name) do |ast, *args|
                    error = error.call(ast, *args) if error.is_a?(Proc)
                    remedy = remedy.call(ast, *args) if remedy.is_a?(Proc)
                    SyntaxError.new(name, ast: ast, args: args, error: error, remedy: remedy)
                end
            end

            syntax_error :unrecognized_character,
                error:  proc { |token| "Unrecognized character #{token.to_s.inspect}." },
                remedy: "Perhaps you meant to put it inside of a string?"

            syntax_error :illegal_octal_digit,
                error:  proc { |token| "Octal literals cannot have 8 or 9 in them: #{token}." },
                remedy: "If you meant to write a decimal number, remove the 0."
            
            syntax_error :missing_right_hand_side_at_eof,
                error:  proc { |token, eof_token| "Missing a value on the right side of \"#{token}\"." },
                remedy: proc { |token, eof_token| "Perhaps you closed the file earlier than intended, or didn't mean to put the - there at all?" }

            # TODO help more with this one. I hate this so much in programs.
            syntax_error :umatched_end_delimiter,
                error:  proc { |token| "Found ending #{token} with no corresponding #{token.end_delimiter.started_by}." },
                remedy: proc { |token| "Perhaps you have too many #{token}'s, or forgot to open with #{token.end_delimiter.started_by}?" }

            # TODO help more with this one. I hate this so much in programs.
            syntax_error :unmatched_start_delimiter,
                error:  proc { |token, closed_by| "#{token} found with no corresponding #{token.start_delimiter.ended_by}." },
                remedy: proc { |token, closed_by| "Perhaps you have too many #{token}'s, or forgot to end with #{token.start_delimiter.ended_by}?"}

            syntax_error :unmatchable_indent,
                error:  proc { |source, new_token, open_indent| "Indents cannot match due to difference in tabs and spaces." },
                remedy: proc { |source, new_token, open_indent| "Either convert tabs to spaces, or vice versa; do not mix them."}

            syntax_error :internal_error,
                error: proc { |token, message| message },
                remedy: proc { |token, message| "Please submit this error to the developer at https://github.com/jkeiser/berg." }
        end
    end
end
