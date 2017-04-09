require_relative "syntax_error"

module Berg
    class Parser
        class SyntaxErrors
            attr_reader :source

            def initialize(source)
                @source = source
            end

            private

            def self.syntax_error(name, error:, remedy:)
                define_method(name) do |ast, *args|
                    error = error.call(source, ast, *args) if error.is_a?(Proc)
                    remedy = remedy.call(source, ast, *args) if remedy.is_a?(Proc)
                    SyntaxError.new(name, source: source, ast: ast, args: args, error: error, remedy: remedy)
                end
            end

            syntax_error :unrecognized_character,
                error:  proc { |source, token| "Unrecognized character #{token.to_s.inspect}." },
                remedy: "Perhaps you meant to put it inside of a string?"

            syntax_error :illegal_octal_digit,
                error:  proc { |source, token| "Octal literals cannot have 8 or 9 in them: #{token}." },
                remedy: "If you meant to write a decimal number, remove the 0."

            # TODO help more with this one. I hate this so much in programs.
            syntax_error :umatched_end_delimiter,
                error:  proc { |source, token| "Found ending #{token} with no corresponding #{token.end_delimiter.started_by}." },
                remedy: proc { |source, token| "Perhaps you have too many #{token}'s, or forgot to open with #{token.end_delimiter.started_by}?" }

            # TODO help more with this one. I hate this so much in programs.
            syntax_error :unmatched_start_delimiter,
                error:  proc { |source, token, closed_by| "#{token} found with no corresponding #{token.start_delimiter.ended_by}." },
                remedy: proc { |source, token, closed_by| "Perhaps you have too many #{token}'s, or forgot to end with #{token.start_delimiter.ended_by}?"}

            syntax_error :unmatchable_indent,
                error:  proc { |source, new_token, open_indent| "Indents cannot match due to difference in tabs and spaces." },
                remedy: proc { |source, new_token, open_indent| "Either convert tabs to spaces, or vice versa; do not mix them."}

            syntax_error :internal_error,
                error: proc { |source, token, message| message },
                remedy: proc { |source, token, message| "Please submit this error to the developer at https://github.com/jkeiser/berg." }
        end
    end
end
