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
                error:  "Unrecognized character.",
                remedy: "Perhaps you meant to put it inside of a string?"

            syntax_error :illegal_octal_digit,
                error:  "Octal literals cannot have 8 or 9 in them.",
                remedy: "If you meant to write a decimal number, remove the 0."
            
            syntax_error :empty_exponent,
                error: "Empty exponent.",
                remedy: "If you meant the \"e\" to have an exponent, add some numbers."

            syntax_error :float_with_trailing_identifier,
                error: "Number is mixed up with a word.",
                remedy: "If you wanted a number, you can remove the word characters. If you're trying to get a property of an integer with \".\", make sure the property name starts with a word character."

            syntax_error :variable_name_starting_with_an_integer,
                error: "Number is mixed up with a word.",
                remedy: "If it's a variable name, change it to start with a character instead of a number. If you wanted a number, you can remove the word characters."

            syntax_error :missing_right_hand_side_at_eof,
                error:  proc { |token, eof_token| "Missing a value on the right side of \"#{token}\"." },
                remedy: proc { |token, eof_token| "Perhaps you closed the file earlier than intended, or didn't mean to put the \"#{token}\" there at all?" }

            syntax_error :missing_left_hand_side_at_sof,
                error:  proc { |token, sof_token| "Missing a value on the left side of \"#{token}\"." },
                remedy: proc { |token, sof_token| "Did you mean for the \"#{token}\" to be there?" }

            syntax_error :prefix_or_infix_in_front_of_infix_operator,
                error: proc { |token, because_of_infix| "Operators \"#{token}\" and \"#{because_of_infix}\" are in the wrong order." },
                remedy: proc { |token, because_of_infix| "Perhaps one of them was an error, or you meant to have a value between them?" }
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
