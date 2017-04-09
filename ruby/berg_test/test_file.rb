require "yaml"

require_relative "test"

module BergTest
    class TestFile
        attr_reader :test_run
        attr_reader :filename
        attr_reader :tests

        def initialize(test_run, filename)
            @test_run = test_run
            @filename = filename
            @tests = []
            path = File.join(test_run.test_path, filename)
            yaml = YAML.load(IO.read(path))
            gather_tests(yaml)
        end

        #
        # Run all tests in the file
        #
        def run
            if tests.any? { |test| test_run.should_run?(test) }
                output.test_file_starting(self)
                tests.each do |test|
                    test.run
                end
                output.test_file_complete(self)
            else
                output.test_file_skipped(self)
            end
        end

        private

        def output
            test_run.output
        end

        #
        # Gather all the tests from the YAML (and give them names)
        #
        def gather_tests(test_yaml, test_name=nil)
            case test_yaml
            when Array
                test_yaml.each_with_index do |child_yaml, index|
                    gather_tests(child_yaml, test_name ? "#{test_name}[#{index+1}]" : index+1)
                end
            when Hash
                if test_yaml["Berg"]
                    test_name ||= 1
                    tests << Test.new(self, test_name, test_yaml)
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
