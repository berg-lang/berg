require "pastel"
require "tty/cursor"
require "tty/table"

module BergTest
    class TestOutput
        # The stream where progress and results will be output
        attr_reader :output_stream

        # Create a new TestOutput
        def initialize(output_stream)
            @output_stream = output_stream

            @pastel = Pastel.new
            @cursor = TTY::Cursor
        end

        # Event called just before a test run starts
        def test_run_starting(test_run)
        end

        # Event called after a test run completes
        def test_run_complete(test_run)
        end

        # Event called when a test file is skipped
        def test_file_skipped(test_file)
        end

        # Event called just before a test run starts
        def test_file_starting(test_file)
            print_progress test_file
        end

        # Event called just after a test file completes
        def test_file_complete(test_file)
            print_results test_file
        end

        # Event called when a test is skipped
        def test_skipped(test)
            print_progress test_file
        end

        # Event called just before a test run starts
        def test_starting(test)
        end

        # Event called just after a test completes
        def test_complete(test)
            print_progress test.test_file
        end

        private

        attr_reader :pastel
        attr_reader :cursor

        def print_progress(test_file)
            results = test_file.tests.group_by { |test| test.result }
            results.delete(nil) # Results that aren't started don't count
            results[:success] ||= []
            results.delete(:skipped)

            output_stream.print cursor.column(0)
            progress = "#{test_file.filename} (#{results[:success].size}/#{test_file.tests.size})"
            # If there is anything but success in there, we failed.
            if results.size > 1
                output_stream.print pastel.red(progress)
            else
                output_stream.print progress
            end
        end

        def print_results(test_file)
            results = test_file.tests.group_by { |test| test.result }
            results.delete(nil) # Results that aren't started don't count
            results[:success] ||= []

            output_stream.print cursor.column(0)

            progress = "#{test_file.filename} ("
            progress << results.map do |type, result|
                "#{result.size} #{type}"
            end.join(", ")
            progress << " out of #{test_file.tests.size} total)"

            skipped = results.delete(:skipped)
            if results.size > 1
                output_stream.puts pastel.red(progress)
                output_stream.puts ""
                table = TTY::Table.new
                test_file.tests.each do |test|
                    next if test.result == :success
                    table << [ pastel.red(test.result), test.test_name, pastel.dim(test.result_message) ]
                end
                output_stream.puts table.render(multiline: true)
                output_stream.puts ""
            else
                output_stream.puts pastel.green(progress)
            end
        end
    end
end