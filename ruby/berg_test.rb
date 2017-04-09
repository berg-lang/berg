require_relative "berg_test/test_run"

test_path = File.expand_path("../../tests", __FILE__)
test_run = BergTest::TestRun.new(test_path, whitelist: ARGV, output_stream: STDOUT)
test_run.run
