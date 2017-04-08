require_relative "berg_test/test_runner"

test_path = File.expand_path("../../tests", __FILE__)
BergTest::TestRunner.new(test_path).run
