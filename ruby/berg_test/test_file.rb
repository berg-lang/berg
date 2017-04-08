require "yaml"

require_relative "test"

module BergTest
    class TestFile
        attr_reader :runner
        attr_reader :filename
        attr_reader :tests

        def initialize(runner, filename)
            @runner = runner
            @filename = filename
            @tests = {}
            path = File.join(runner.test_path, filename)
            yaml = YAML.load(IO.read(path))
            gather_tests(yaml)
        end

        #
        # Run all tests in the file
        #
        def run
            tests.each_value do |test|
                test.run
            end
        end

        #
        # Gather all the tests from the YAML (and give them names)
        #
        def gather_tests(test_yaml, test_name=nil)
            case test_yaml
            when Array
                test_yaml.each_with_index do |child_yaml, index|
                    gather_tests(child_yaml, test_name ? "#{test_name}[#{index}]" : index)
                end
            when Hash
                if test_yaml["Berg"]
                    test_name ||= 0
                    tests[test_name] = Test.new(self, test_name, test_yaml)
                else
                    test_yaml.each do |child_name,child_yaml|
                        gather_tests(child_yaml, test_name ? "#{name}.#{child_name}" : child_name)
                    end
                end
            else
                raise "Not a test or test list: #{test_yaml.inspect}"
            end
        end
    end
end
