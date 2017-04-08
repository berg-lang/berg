module BergTest
    class TestResult < StandardError
        attr_reader :property_name
        attr_reader :result_message

        def initialize(property_name, result_message)
            @property_name = property_name
            @result_message = result_message
        end
    end
    class TestFailure < TestResult
        def result_type
            :failure
        end
    end
    class BadTest < TestResult
        def result_type
            :bad_test
        end
    end
end
