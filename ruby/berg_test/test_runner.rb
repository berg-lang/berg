require "pastel"
require "tty/cursor"
require_relative "test_file"

module BergTest
    class TestRunner
        attr_reader :test_path
        attr_reader :test_files
        attr_reader :output

        def initialize(test_path, output: STDOUT)
            @test_path = test_path
            @output = output
            @test_files = {}
            @pastel = Pastel.new
            @cursor = TTY::Cursor
            gather_test_files
        end

        def run
            test_files.each do |filename, test_file|
                output.print "#{filename} "
                print_progress test_file
                test_file.run
                print_results test_file
            end
        end

        # Event called when a test is started
        def starting(test)
        end

        # Event called when a test is complete
        def complete(test)
            print_progress test.test_file
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

        def print_progress(test_file)
            results = test_file.tests.values.group_by { |test| test.result }
            results.delete(nil) # Results that aren't started don't count
            results[:success] ||= []

            output.print cursor.column(0)
            output.print test_file.filename
            output.print " "
            progress = "(#{results[:success].size}/#{test_file.tests.size})"
            # If there is anything but success in there, we failed.
            if results.size > 1
                output.print pastel.red(progress)
            else
                output.print pastel.green(progress)
            end
        end

        def print_results(test_file)
            results = test_file.tests.values.group_by { |test| test.result }
            results.delete(nil) # Results that aren't started don't count
            results[:success] ||= []

            output.print cursor.column(0)

            progress = "#{test_file.filename} ("
            progress << results.map do |type, result|
                "#{result.size} #{type}"
            end.join(", ")
            progress << " out of #{test_file.tests.size} total)"

            if results.size > 1
                output.puts pastel.red(progress)
                output.puts ""
                test_file.tests.each do |test_name, test|
                    next if test.result == :success
                    output.puts "    #{test_name}: #{pastel.red(test.result)} - #{test.result_message}"
                end

                output.puts ""
            else
                output.puts pastel.green(progress)
            end
        end
    end
end