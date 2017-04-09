require_relative "test_file"
require_relative "test_output"

module BergTest
    class TestRun
        attr_reader :test_path
        attr_reader :test_files
        attr_reader :whitelist
        attr_reader :output

        def initialize(test_path, whitelist: [], output_stream: STDOUT)
            @test_path = test_path
            @whitelist = whitelist unless whitelist.empty?
            @output = TestOutput.new(output_stream)
            @test_files = {}
            gather_test_files
        end

        def run
            output.test_run_starting(self)
            test_files.each do |filename, test_file|
                test_file.run
            end
            output.test_run_complete(self)
        end

        def should_run?(test)
            return true if !whitelist
            whitelist.any? { |match_string| test.test_name.index(match_string) || test.test_file.filename.index(match_string) }
        end

        private

        # Pastel (pastel.red.bold("hi"))
        attr_reader :pastel
        # Cursor (cursor.save/restore)
        attr_reader :cursor

        #
        # Gather the list of test files (all .yaml files under the test root).
        #
        def gather_test_files(path=nil)
            full_path = path ? File.join(test_path, path) : test_path
            if File.directory?(full_path)
                Dir.entries(full_path).each do |filename|
                    next if [".", ".."].include?(filename)
                    child_path = path ? File.join(path, filename) : filename
                    gather_test_files(child_path)
                end
            else
                test_files[path] = TestFile.new(self, path) if path && File.extname(path) == ".yaml"
            end
        end

    end
end