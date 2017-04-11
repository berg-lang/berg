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

                let :parsed_expression_root do
                    parser = BergLang::Parser.new(source)
                    parser.parse
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

                let :parsed_expression do
                    # Strip off the outer DelimitedOperation before checking the AST (since it's always the same)
                    expect(parsed_expression_root).to be_a BergLang::Expressions::DelimitedOperation
                    expect(parsed_expression_root.start_delimiter.key).to eq :sof
                    expect(parsed_expression_root.end_delimiter.key).to eq :eof
                    parsed_expression_root.expression
                end

                if test_spec["Ast"]
                    generate_ast_tests([], test_spec["Ast"])
                end
                if test_spec["Error"]
                    generate_error_tests(test_spec["Error"])
                end
            end
        end

        def generate_error_tests(error_spec)
            expected_range, expected_error = error_spec
            return unless expected_range && expected_range != ""
            if expected_error
                error_description = expected_error
                if error_description.size > MAX_TEST_DESCRIPTION_SOURCE_SIZE
                    error_description = "#{error_description[0...MAX_TEST_DESCRIPTION_SOURCE_SIZE-3]}..."
                end
                it "has an error with text #{error_description}" do
                    expect("#{syntax_error.error} #{syntax_error.remedy}").to eq expected_error
                end
            end
            if expected_range
                it "has an error at row/column #{expected_range}" do
                    expected_range_end, expected_range_begin = parse_range(expected_range)

                    actual_range_begin = syntax_error.source_range.begin_location
                    actual_range_end = syntax_error.source_range.end_location

                    expect(actual_range_begin).to eq(expected_range_begin)
                    expect(actual_range_end).to eq(expected_range_end)
                end
            end
        end

        def generate_ast_tests(property_path, ast_spec)
            case ast_spec
            when Hash
                if ast_spec.size != 1
                    raise "Expected ast #{property_path.join(".")} to have exactly one key (Type: ExpectedValue), but it has #{ast_spec.size}!"
                end
                expected_type, expected_value = ast_spec.first
            else
                expected_value = ast_spec
            end

            # Simpler test specs: if test is "IntegerLiteral: [1@1-3, 12]", don't look for sub-properties
            case expected_value
            when Array, String
                expected_value = { "$Range" => expected_value }
            end

            expected_range = expected_value["$Range"]
            expected_string = expected_value["$String"]
            expected_range, expected_string = expected_range if expected_range.is_a?(Array)

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

                if expected_string
                    it "has string \"#{expected_string}\"" do
                        actual_string = expression.source_range.string
                        expect(actual_string).to eq expected_string
                    end
                end

                if expected_range
                    it "has row/column range #{expected_range}" do
                        expected_range_begin, expected_range_end = parse_range(expected_range)

                        actual_range_begin = expression.source_range.begin_location
                        actual_range_end = expression.source_range.end_location

                        expect(actual_range_begin).to eq(expected_range_begin)
                        expect(actual_range_end).to eq(expected_range_end)
                    end
                end
            end

            # Child properties
            expected_value.each do |child_property_name, expected_property_value|
                next if %w($Range $String).include?(child_property_name)
                generate_ast_tests(property_path + [ child_property_name ], expected_property_value)
            end
        end

        def to_range_string(range)
            range_begin, range_end = range

            if range_begin[0] == range_end[0]
                if range_begin[1] == range_end[1] + 1
                    "#{range_begin[0]}@#{range_begin[1]}"
                else
                    "#{range_begin[0]}@#{range_begin[1]}-#{range_end[1]}"
                end
            else
                "#{range_begin[0]}@#{range_begin[1]}-#{range_end[0]}@#{range_end[1]}"
            end
        end

        module InstanceMethods
            def parse_range(range)
                expect(range).to match /^(\d+)@(\d+)(-(\d+@)?(\d+)|(\+0))?$/
                range =~ /^(\d+)@(\d+)(-(\d+@)?(\d+)|(\+0))?$/

                range_begin = [$1.to_i, $2.to_i]
                if $6
                    range_end = nil
                elsif $5
                    range_end = [$4 ? $4.to_i : range_begin[0], $5 ? $5.to_i : range_begin[1]]
                else
                    range_end = range_begin
                end
                [ range_begin, range_end ]
            end

            def to_snake_case(string)
                string.gsub(/([A-Z]+)([A-Z][a-z])/, '\1_\2').
                        gsub(/([a-z])([A-Z])/, '\1_\2').
                        downcase
            end
        end

    end
end
