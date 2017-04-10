require "rspec"
require "yaml"
require_relative "berg/source"
require_relative "berg/parser"

test_root = File.expand_path("../../spec_tests", __FILE__)

RSpec.configure do |config|
  config.filter_run :focus => true
  config.run_all_when_everything_filtered = true

  # Explicitly disable :should syntax
  config.expect_with :rspec do |c|
    c.syntax = :expect
  end
  config.mock_with :rspec do |c|
    c.syntax = :expect
  end
end

RSpec.describe "Berg Specs" do
    def self.generate_tests_from_path(path)
        context File.basename(path) do
            Dir.entries(path).each do |filename|
                next if [".", ".."].include?(filename)
                child_path = File.join(path, filename)
                if File.directory?(child_path)
                    generate_tests_from_path(child_path)
                elsif File.extname(child_path) == ".yaml"
                    test_spec = YAML.load(IO.read(child_path))
                    context File.basename(child_path[0..-6]) do
                        generate_tests_from_spec(test_spec)
                    end
                end
            end
        end
    end

    def self.generate_tests_from_spec(test_spec)
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

    MAX_SOURCE_DESCRIPTION_SIZE = 80

    def self.generate_test(test_spec)
        source_description = test_spec["Berg"]
        if source_description.size > MAX_SOURCE_DESCRIPTION_SIZE
            source_description = "#{source_description[0...(MAX_SOURCE_DESCRIPTION_SIZE-3)]}..."
        end
        context "When Berg source is #{test_spec["Berg"]}" do
            let :source do
                Berg::Source.new("spec_test", test_spec["Berg"])
            end

            let :parsed_expression_root do
                parser = Berg::Parser.new(source)
                parser.parse
            end

            let :parsed_expression do
                # Strip off the outer DelimitedOperation before checking the AST (since it's always the same)
                expect(parsed_expression_root).to be_a Berg::Expressions::DelimitedOperation
                expect(parsed_expression_root.start_delimiter.key).to eq :sof
                expect(parsed_expression_root.end_delimiter.key).to eq :eof
                parsed_expression_root.expression
            end

            if test_spec["Ast"]
                generate_ast_tests([], test_spec["Ast"])
            end
        end
    end

    def self.generate_ast_tests(property_path, ast_spec)
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
                expected_class = eval("Berg::Expressions::#{expected_type}")
                it "is #{expected_type}" do
                    expect(expression).to be_a expected_class
                end
            end
            if expected_string
                it "has string \"#{expected_string}\"" do
                    actual_string = source.substr(*expression.input_range)
                    expect(actual_string).to eq expected_string
                end
            end

            if expected_range
                it "has row/column range #{expected_range}" do
                    expected_range_start, expected_range_end = parse_range(expected_range)

                    actual_range_start = source.location(expression.input_range[0])
                    actual_range_end = source.location(expression.input_range[1])

                    expect(actual_range_start).to eq(expected_range_start)
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

    def parse_range(range)
        expect(range).to match /^(\d+)@(\d+)(-(\d+@)?(\d+))?$/
        range =~ /^(\d+)@(\d+)(-(\d+@)?(\d+))?$/

        range_start = [$1.to_i, $2.to_i]
        range_end = [$4 ? $4.to_i : $1.to_i, $5 ? $5.to_i : $2.to_i+1]
        [ range_start, range_end ]
    end

    def self.to_range_string(range)
        range_start, range_end = range

        if range_start[0] == range_end[0]
            if range_start[1] == range_end[1] + 1
                "#{range_start[0]}@#{range_start[1]}"
            else
                "#{range_start[0]}@#{range_start[1]}-#{range_end[1]}"
            end
        else
            "#{range_start[0]}@#{range_start[1]}-#{range_end[0]}@#{range_end[1]}"
        end
    end

    def to_snake_case(string)
        string.gsub(/([A-Z]+)([A-Z][a-z])/, '\1_\2').
                gsub(/([a-z])([A-Z])/, '\1_\2').
                downcase
    end

    generate_tests_from_path(test_root)
end
