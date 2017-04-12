require "yaml"
require "berg_lang/source"
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
        MAX_TEST_DESCRIPTION_SOURCE_SIZE = 80

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
                            generate_tests_from_spec(test_spec)
                        end
                    end
                end
            end
        end

        def generate_tests_from_spec(test_spec)
            case test_spec
            when Array
                test_spec.each_with_index do |child_spec, index|
                    generate_tests_from_spec(child_spec)
                end

            when Hash
                if test_spec["Berg"]
                    # We have a test! Generate it.
                    generate_test(test_spec)
                else
                    test_spec.each do |child_name,child_spec|
                        context child_name do
                            generate_tests_from_spec(child_spec)
                        end
                    end
                end
            else
                raise "Not a test or test list: #{test_spec.inspect}"
            end
        end

        def generate_test(test_spec)
            source_description = test_spec["Berg"]
            if source_description.size > MAX_TEST_DESCRIPTION_SOURCE_SIZE
                source_description = "#{source_description[0...(MAX_TEST_DESCRIPTION_SOURCE_SIZE-3)]}..."
            end

            context "When Berg source is #{test_spec["Berg"]}" do
                let :source do
                    BergLang::Source.new("spec_test", test_spec["Berg"])
                end

                let :syntax_error do
                    begin
                        parser = BergLang::Parser.new(source)
                        expression = parser.parse
                        raise "Expected a parse error, but no error happened! Instead, the expression #{expression} was returned."
                    rescue BergLang::Parser::SyntaxError
                        return $!
                    end
                end

                let :parsed_expression_root do
                    parser = BergLang::Parser.new(source)
                    parser.parse
                end

                let :parsed_expression do
                    # Strip off the outer DelimitedOperation before checking the AST (since it's always the same)
                    expect(parsed_expression_root).to be_a BergLang::Expressions::DelimitedOperation
                    expect(parsed_expression_root.start_delimiter.key).to eq :sof
                    expect(parsed_expression_root.end_delimiter.key).to eq :eof
                    parsed_expression_root.expression
                end

                if test_spec["Ast"]
                    generate_ast_tests([], test_spec["Ast"], BergLang::Source.new("spec_test", test_spec["Berg"]))
                end
                if test_spec["Error"]
                    generate_error_tests(test_spec["Error"], BergLang::Source.new("spec_test", test_spec["Berg"]))
                end
            end
        end

        def generate_ast_tests(property_path, ast_spec, source)
            case ast_spec
            when Hash
                if ast_spec.size != 1
                    raise "Expected ast #{property_path.join(".")} to have exactly one key (Type: ExpectedValue), but it has #{ast_spec.size}!"
                end
                expected_type, expected_value = ast_spec.first
            else
                expected_value = ast_spec
            end

            expected_range, expected_term = parse_range(expected_value, source)

            if property_path.any?
                description = "the parsed expression property #{property_path.join(".")}"
            else
                description = "the parsed expression"
            end

            context description do
                let :expression do
                    property_value = parsed_expression
                    property_path.each do |property_name|
                        property_value = property_value.send(to_snake_case(property_name))
                    end
                    property_value
                end

                if expected_type
                    expected_class = eval("BergLang::Expressions::#{expected_type}")
                    it "is #{expected_type}" do
                        expect(expression).to be_a expected_class
                    end
                end

                if expected_term
                    it "has string \"#{expected_term}\"" do
                        actual_string = expression.source_range.string
                        expect(actual_string).to eq expected_term
                    end
                end

                if expected_range
                    it "has row/column range #{to_range_string(expected_range)}" do
                        expect(expression.source_range.begin_location).to eq(expected_range.begin_location)
                        expect(expression.source_range.end_location).to eq(expected_range.end_location)
                    end
                end
            end

            # Child properties
            if expected_value.is_a?(Hash)
                expected_value.each do |child_property_name, expected_property_value|
                    next if %w($Range $Term).include?(child_property_name)
                    generate_ast_tests(property_path + [ child_property_name ], expected_property_value, source)
                end
            end
        end

        def generate_error_tests(error_spec, source)
            # Read the error spec
            # 1@1-2=<error>
            # 1@1-2@3=<error>
            # 1@1=<error>
            # <term>=<error>
            # <error>
            # <highlighted string>=<error>
            # { Range:, Term:, Error: }
            expected_range, expected_term, expected_error = parse_error_spec(error_spec, source)
            if expected_error
                error_description = expected_error
                if error_description.size > MAX_TEST_DESCRIPTION_SOURCE_SIZE
                    error_description = "#{error_description[0...MAX_TEST_DESCRIPTION_SOURCE_SIZE-3]}..."
                end
                it "has an error with text #{error_description}" do
                    expect("#{syntax_error.error} #{syntax_error.remedy}").to eq expected_error
                end
            end
            if expected_term
                it "has an error reported against #{expected_term.inspect}" do
                    expect(syntax_error.source_range.string).to eq(expected_term)
                end
            end
            if expected_range
                it "has an error at row/column #{to_range_string(expected_range)}" do
                    expect(syntax_error.source_range.begin_location).to eq(expected_range.begin_location)
                    expect(syntax_error.source_range.end_location).to eq(expected_range.end_location)
                end
            end
        end

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
                raise "Unexpected type #{error_spec.string} for error_spec #{error_spec} in test with Berg source #{source.string.inspect}"
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

        def parse_range(range_spec, source)
            case range_spec
            when String
                range, term = range_spec.split(" = ", 2)

                if range =~ /^(\d+)@(\d+)(-(\d+@)?(\d+)|(\+0))?$/
                    begin_line = $1.to_i
                    begin_column = $2.to_i
                    end_line = $4 ? $4.to_i : begin_line
                    end_column = $5 ? $5.to_i : begin_column
                    range = LocationSourceRange.new(source, [begin_line, begin_column], [end_line, end_column])

                # If range does not work, then we assume the whole thing is a term and the = sign (if any) was a red herring.
                else
                    term = range_spec
                    range = nil

                    # Check if the term exists more than once in the source, and force the user to specify it if so.
                    index1 = source.string.index(term)
                    if index1
                        if index2 = source.string.index(term, index1+1)
                            location1 = to_range_string(BergLang::SourceRange.new(source, index1, index1+term.length))
                            location2 = to_range_string(BergLang::SourceRange.new(source, index2, index2+term.length))
                            raise "#{term} exists more than once in source #{source.string.inspect}. Please specify which one using range syntax #{location1} or #{location2}."
                        end
                    end
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

            def to_snake_case(string)
                string.gsub(/([A-Z]+)([A-Z][a-z])/, '\1_\2').
                        gsub(/([a-z])([A-Z])/, '\1_\2').
                        downcase
            end
        end

    end
end
