require_relative "../berg/source"
require_relative "../berg/parser"
require_relative "errors"

module BergTest
    class Test
        attr_reader :test_file
        attr_reader :test_name
        attr_reader :test
        attr_reader :result
        attr_reader :result_message

        def initialize(test_file, test_name, test)
            @test_file = test_file
            @test_name = test_name
            @test = test
        end

        def output
            test_file.test_run.output
        end

        def should_run?
            test_file.test_run.should_run?(self)
        end

        #
        # Run a single test.
        #
        def run
            if !should_run?
                output.test_skipped(self)
                @result = :skipped
                return
            end

            output.test_starting(self)
            begin
                expression = parse(test_name, test["Berg"])
                expression = strip_outer_expression(expression)
                test_expression_ast("Ast", expression, test["Ast"])
                @result = :success
            rescue TestFailure, BadTest
                @result = $!.result_type
                @result_message = $!.result_message
            ensure
                output.test_complete(self)
            end
        end

        private

        attr_reader :source

        def parse(test_name, berg_string)
            begin
                raise BadTest.new("Berg", "Berg value is not a string: #{berg_string.inspect}") unless berg_string.is_a?(String)
                @source = Berg::Source.new(test_name, berg_string)
                parser = Berg::Parser.new(source)
                parser.parse
            rescue
                raise TestFailure.new("Berg", "Compiler error: #{$!}\n#{$!.backtrace.join("\n")}")
            end
        end

        def strip_outer_expression(expression)
            if !expression.is_a?(Berg::Expressions::DelimitedOperation)
                raise TestFailure.new("Ast", "Compiler bug: Parsed expression is not a DelimitedOperation. Instead it was #{expression.class}.")
            end
            if expression.start_delimiter.key != :sof || expression.end_delimiter.key != :eof
                raise TestFailure.new("Ast", "Compiler bug: Parsed expression is not a :sof / :eof delimited operation. Instead it was #{expression.start_delimiter} / #{expression.end_delimiter}")
            end
            expression.expression
        end

        def test_expression_ast(property_name, expression, expected)
            if !expected.is_a?(Hash)
                raise BadTest.new(property_name, "Ast is a #{expected.class}, expected Hash!")
            end
            if expected.keys.size != 1
                raise BadTest.new(property_name, "Ast has #{expected.keys.size}, expected 1!")
            end

            # Check whether it has the right ast type
            expected_type, expected_value = expected.first
            if !eval("expression.is_a?(Berg::Expressions::#{expected_type})")
                raise TestFailure.new(property_name, "Parsed expression is #{expression.class}, expected #{expected_type}")
            end

            # Check sub-properties and values
            test_ast_value(property_name, expression, expected_value)
        end

        def test_ast_value(property_name, expression, expected_value)
            case expected_value
            when Hash
                expected_value.each do |property, expected_property_value|
                    if property == "$Range"
                        test_ast_value(property_name, expression, expected_property_value)
                    else
                        property_value = expression.send(to_snake_case(property))
                        test_ast_value("#{property_name}.#{property}", property_value, expected_property_value)
                    end
                end
            when Array
                range, string = expected_value
                test_ast_value(property_name, expression, string)
                actual_range = [ source.location(expression.input_range[0]), source.location(expression.input_range[1]) ]
                expected_range = parse_range(property_name, range)
                if actual_range != expected_range
                    raise TestFailure.new(property_name, "Expression range is #{to_range_string(actual_range)}, expected #{range}!")
                end
            when String
                actual_value = source.substr(*expression.input_range)
                if actual_value != expected_value
                    raise TestFailure.new(property_name, "Expression string is #{actual_value}, expected #{expected_value}!")
                end
            else
                raise "Unexpected expected value #{expected_value.class} #{expected_value}"
            end
        end

        def parse_range(property_name, string)
            if string !~ /^(\d+)@(\d+)(-(\d+@)?(\d+))?$/
                raise BadTest.new(property_name, "Invalid range string #{string.inspect}. Should be something like 10@3 (<row>@<column>), 10@3-5, or 10@3-11@1.") 
            end

            range_start = [$1.to_i, $2.to_i]
            range_end = [$4 ? $4.to_i : $1.to_i, $5 ? $5.to_i : $2.to_i+1]
            [ range_start, range_end ]
        end

        def to_range_string(range)
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

    end
end
