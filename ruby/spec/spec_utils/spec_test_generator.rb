require "yaml"
require "berg_lang/source_string"
require "berg_lang/parser"

module SpecUtils
    #
    # Generates tests from the top-level Berg spec tests.
    #
    # Goes through each YAML file, iterates through the tests and generates rspec for each.
    #
    module SpecTestGenerator
        def self.extended(other)
            other.include(InstanceMethods)
        end

        # Maximum length of Berg source code in a test descriptino
        TRUNCATE_SIZE = 80

        def generate_tests_from_path(path)
            context File.basename(path) do
                Dir.entries(path).each do |filename|
                    next if [".", ".."].include?(filename)
                    child_path = File.join(path, filename)
                    if File.directory?(child_path)
                        generate_tests_from_path(child_path)
                    elsif File.extname(child_path) == ".yaml"
                        test_spec = YAML.load(IO.read(child_path), child_path)
                        context File.basename(child_path[0..-6]) do
                            generate_tests_from_spec(child_path, test_spec)
                        end
                    end
                end
            end
        end

        def generate_tests_from_spec(path, test_spec)
            case test_spec
            when Array
                test_spec.each_with_index do |child_spec, index|
                    generate_tests_from_spec("#{path}[#{index}]", child_spec)
                end

            when Hash
                if test_spec["Berg"]
                    # We have a test! Generate it.
                    generate_test(path, test_spec)
                else
                    test_spec.each do |child_name,child_spec|
                        context child_name do
                            generate_tests_from_spec("#{path}.#{child_name}", child_spec)
                        end
                    end
                end
            else
                raise "Not a test or test list: #{test_spec.inspect}"
            end

        rescue
            STDERR.puts "ERROR: in test #{path}"
            raise
        end

        def generate_test(path, test_spec)
            test_spec.each do |key, expected_value|
                test_type, expected_key = key.split(/\s*->\s*/, 2)
                case test_type
                when "Ast"
                    generate_ast_tests([], expected_key, expected_value, test_spec)
                when "Error"
                    generate_error_tests(expected_value, test_spec)
                when "Berg", "Result"
                else
                    raise "Unexpected test key #{key}! Expected Berg, Ast, Error, or Result."
                end
            end

        rescue Object
            STDERR.puts "ERROR: in test with spec:"
            require "pp"
            pp test_spec
            raise
        end

        def generate_ast_tests(property_path, expected_type, expected_value, test_spec)
            ast_tests = ast_test_specs(property_path, expected_type, expected_value, test_spec)
            test_descriptions = ast_tests.map do |property_path, expected_type, expected_range, expected_term|
                property_description = [ "Ast", *property_path ].join(".")

                descriptions = []
                descriptions << "a #{expected_type}" if expected_type
                descriptions << "\"#{expected_term}\"" if expected_term
                descriptions << " at #{to_range_string(expected_range)}" if expected_range
                if descriptions.any?
                    "#{property_description} is #{descriptions.join(" ")}"
                end
            end.reject { |desc| desc.nil? }

            it "When Berg source is #{source_description(test_spec)}, #{english_join(test_descriptions, "and")}" do
                # Parse.
                parsed_expression = nil
                begin
                    parser_output.indented do
                        parsed_expression = parse_expression(test_spec)
                    end
                    ast_tests.each do |property_path, expected_type, expected_range, expected_term|

                        begin
                            expression = parsed_expression
                            property_path.each do |property_name|
                                expression = expression.send(to_snake_case(property_name))
                            end

                            # Check the results
                            if expected_type
                                expect(expression).to be_a eval("BergLang::Ast::#{expected_type}")
                            end

                            if expected_term
                                expect(expression.source_range.string).to eq expected_term
                            end

                            if expected_range
                                expect(expression.source_range.begin_location).to eq(expected_range.begin_location)
                                expect(expression.source_range.end_location).to eq(expected_range.end_location)
                            end
                        rescue Object
                            STDERR.puts "Error in #{[ "Ast", *property_path ].join(".")}"
                            raise
                        end
                    end
                rescue Object
                    STDERR.puts "Parsed expression: #{parsed_expression}" if parsed_expression
                    STDERR.puts "Parser output:\n#{parser_output.stream.string}"
                    STDERR.puts "--------------"
                    raise
                end
            end
        end

        # Gather all desired tests, recursively
        def ast_test_specs(property_path, expected_type, expected_value, test_spec)
            expected_range, expected_term = parse_range(expected_value || "", create_source(test_spec))
            results = []
            results << [ property_path, expected_type, expected_range, expected_term ]

            # Recurse to child properties
            if expected_value.is_a?(Hash)
                expected_value.each do |key, expected_property_value|
                    child_property_name, expected_property_type = key.split(/\s*->\s*/, 2)
                    next if %w($Range $Term).include?(child_property_name)
                    results += ast_test_specs(property_path + [ child_property_name ], expected_property_type, expected_property_value, test_spec)
                end
            end
            results
        end

        def generate_error_tests(error_spec, test_spec)
            expected_range, expected_term, expected_error = parse_error_spec(error_spec, test_spec)

            test_descriptions = []
            test_descriptions << "text #{truncate(expected_error)}" if expected_error
            test_descriptions << "reported against #{expected_term.inspect}" if expected_term
            test_descriptions << "row/column #{to_range_string(expected_range)}" if expected_range

            it "When Berg source is #{source_description(test_spec)}, the parser emits an error with #{english_join(test_descriptions, "and")}" do

                parsed_expression = nil
                begin
                    # Parse and grab the error.
                    syntax_error = nil
                    begin
                        parser_output.indented do
                            parsed_expression = parse_expression(test_spec)
                        end
                        raise "Expected a parse error, but no error happened! Instead, the expression #{parsed_expression} was returned."
                    rescue BergLang::SyntaxError
                        syntax_error = $!
                    end

                    # Check its properties.
                    if expected_error
                        expect("#{syntax_error.error} #{syntax_error.remedy}").to eq expected_error
                    end
                    if expected_term
                        expect(syntax_error.source_range.string).to eq(expected_term)
                    end
                    if expected_range
                        expect(syntax_error.source_range.begin_location).to eq(expected_range.begin_location)
                        expect(syntax_error.source_range.end_location).to eq(expected_range.end_location)
                    end
                rescue Object
                    STDERR.puts "Parsed expression: #{parsed_expression}" if parsed_expression
                    STDERR.puts "Parser output:\n#{parser_output.stream.string}"
                    STDERR.puts "--------------"
                    raise
                end
            end
        end

        def create_source(test_spec)
            BergLang::StringSource.new("spec_test", test_spec["Berg"])
        end

        # Read the error spec
        # 1@1-2=<error>
        # 1@1-2@3=<error>
        # 1@1=<error>
        # 1@1+0=<error>
        # <term>=<error>
        # <error>
        # <highlighted string>=<error>
        # { Range:, Term:, Error: }
        def parse_error_spec(error_spec, source)
            case error_spec
            when String
                range, error = error_spec.split(" = ",2)
                if error
                    range, term = parse_range(range, source)
                    [ range, term, nil ]
                else
                    [ nil, nil, error_spec ]
                end
            when Hash
                [ error_spec["$Range"], error_spec["$Term"], error_spec["$Error"] ]
            else
                raise "Unexpected type #{error_spec.class} for error_spec #{error_spec} in test with Berg source #{source.string.inspect}"
            end

        end

        def to_range_string(range)
            if range.begin_line == range.end_line
                if range.begin_column == range.end_column + 1
                    "#{range.begin_line}@#{range.begin_column}"
                else
                    "#{range.begin_line}@#{range.begin_column}-#{range.end_column}"
                end
            else
                "#{range.begin_line}@#{range.begin_column}-#{range.end_line}@#{range.end_column}"
            end
        end

        def english_join(parts, join_word)
            case parts.size
            when 0, 1
                parts[0]
            when 2
                "#{parts[0]} #{join_word} #{parts[1]}"
            else
                # Oxford comma yo
                [ parts[0..-2], "#{join_word} #{parts[-1]}" ].join(", ")
            end
        end

        def source_description(test_spec)
            truncate(test_spec["Berg"])
        end

        def truncate(value)
            if value.size <= TRUNCATE_SIZE
                value
            else
                "#{value[0...(TRUNCATE_SIZE-3)]}..."
            end
        end

        def parse_range(range_spec, source)
            case range_spec
            when String
                range, term = range_spec.split(" = ", 2)

                if range =~ /^(\d+)@(\d+)(-(\d+@)?(\d+)|(\+0))?$/
                    begin_line = $1.to_i
                    begin_column = $2.to_i
                    begin_location = [ begin_line, begin_column ]
                    if $6
                        end_location = nil
                    else
                        end_line = $4 ? $4.to_i : begin_line
                        end_column = $5 ? $5.to_i : begin_column
                        end_location = [end_line, end_column]
                    end
                    range = LocationSourceRange.new(source, begin_location, end_location)

                # If range does not work, then we assume the whole thing is a term and the = sign (if any) was a red herring.
                else
                    term = range_spec
                    range = nil

                    # # Check if the term exists more than once in the source, and force the user to specify it if so.
                    # index1 = source.string.index(term)
                    # if index1
                    #     if index2 = source.string.index(term, index1+1)
                    #         location1 = to_range_string(BergLang::SourceRange.new(source, index1, index1+term.length))
                    #         location2 = to_range_string(BergLang::SourceRange.new(source, index2, index2+term.length))
                    #         raise "#{term} exists more than once in source #{source.string.inspect}. Please specify which one using range syntax #{location1} or #{location2}."
                    #     end
                    # end
                end

                [ range, term ]
            when Hash
                [ range_spec["$Range"], range_spec["$Term"] ]
            else
                raise "Unexpected type #{range_spec.class} for expected value #{range_spec} in #{source.string.inspect} test!"
            end
        end

        # Just like SourceRange, except you initialize it with a location and it lazily calculates the index (later).
        class LocationSourceRange < BergLang::SourceRange
            attr_reader :begin_location
            attr_reader :end_location

            def initialize(source, begin_location, end_location)
                @source = source
                @begin_location = begin_location
                @end_location = end_location
            end

            def begin
                source.to_index(begin_location)
            end

            def end
                source.to_index(end_location)
            end
        end

        module InstanceMethods

            def create_source(test_spec)
                self.class.create_source(test_spec)
            end

            def parser_output
                @parser_output ||= BergLang::Output.new(StringIO.new)
            end

            def parse_expression(test_spec)
                parser = BergLang::Parser.new(create_source(test_spec), output: parser_output)
                parsed_expression_root = parser.parse
                # Strip off the outer DelimitedOperation before checking the AST (since it's always the same)
                expect(parsed_expression_root).to be_a BergLang::Ast::DelimitedOperation
                expect(parsed_expression_root.open.key).to eq :sof
                expect(parsed_expression_root.close.key).to eq :eof

                # Get the desired expression
                parsed_expression_root.expression
            end

            def to_snake_case(string)
                string.gsub(/([A-Z]+)([A-Z][a-z])/, '\1_\2').
                        gsub(/([a-z])([A-Z])/, '\1_\2').
                        downcase
            end
        end

    end
end
