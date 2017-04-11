require "rspec"
require_relative "spec_utils/spec_test_generator"

repository_root = File.expand_path("../../..", __FILE__)
test_root = File.join(repository_root, "spec_tests")

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
    extend SpecUtils::SpecTestGenerator
    generate_tests_from_path(test_root)
end
