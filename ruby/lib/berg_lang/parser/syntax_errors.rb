require_relative "../syntax_error"

module BergLang
    class Parser
        module SyntaxErrors
            def output
                parser.output
            end

            # The below methods *generate* the actual public syntax error methods
            private

            def self.syntax_error(name, error:, remedy:)
                define_method(name) do |ast, *args|
                    error_message = error
                    error_message = error_message.call(ast, *args) if error.is_a?(Proc)
                    remedy_message = remedy
                    remedy_message = remedy_message.call(ast, *args) if remedy.is_a?(Proc)
                    SyntaxError.new(name, ast: ast, args: args, error: error_message, remedy: remedy_message)
                end
            end

            syntax_error :unclosed_string,
                error:  "Unclosed string.",
                remedy: "Put a \" at the end to fix this; it is possible, however, that a previous string is the problem. You may need to scan the file. Sorry about that."

            syntax_error :unrecognized_character,
                error:  proc { |token| "Unrecognized character #{token.to_s.inspect}." },
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

            syntax_error :float_without_leading_zero,
                error: "Floating point number found without leading zero.",
                remedy: "Add a 0 before the \".\"."

            syntax_error :variable_name_starting_with_an_integer,
                error: "Number is mixed up with a word.",
                remedy: "If it's a variable name, change it to start with a character instead of a number. If you wanted a number, you can remove the word characters."

            syntax_error :missing_value_between_operators,
                error: proc { |a, b|
                    if [:sof,:indent].include?(a.key)
                        "No value before \"#{b}\"!"
                    elsif [:eof,:undent].include?(b.key)
                        "No value after \"#{a}\"!"
                    else
                        "No value between \"#{a}\" and \"#{b}\"!"
                    end
                },
                remedy: proc { |a, b|
                    if [:sof,:indent].include?(a.key)
                        "Did you mean to put a value or variable there?"
                    elsif [:eof,:undent].include?(b.key)
                        "Did you mean to put a value or variable there?"
                    else
                        "Did you mean to put a value or variable there? Or perhaps they are in the wrong order, or one of them is mistyped."
                    end
                }

            # TODO help more with this one. I hate this so much in programs.
            syntax_error :umatched_close,
                error:  proc { |token| "Found ending #{token} with no corresponding #{token.close.opened_by}." },
                remedy: proc { |token| "Perhaps you have too many #{token}'s, or forgot to open with #{token.close.opened_by}?" }

            # TODO help more with this one. I hate this so much in programs.
            syntax_error :unmatched_close,
                error:  proc { |token, closed_by| "#{token} found with no corresponding #{token.close.closed_by}." },
                remedy: proc { |token, closed_by| "Perhaps you have too many #{token}'s, or forgot to end with #{token.close.closed_by}?"}

            syntax_error :unmatchable_indent,
                error:  proc { |token, open_indent| "Indents cannot match due to difference in tabs and spaces." },
                remedy: proc { |token, open_indent| "Either convert tabs to spaces, or vice versa; do not mix them."}

            syntax_error :internal_error,
                error: proc { |token, message| message },
                remedy: proc { |token, message| "Please submit this error to the developer at https://github.com/jkeiser/berg." }
        end
    end
end
