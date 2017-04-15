#!ruby
require "bundler/setup"
require "berg_lang/parser/operator_list"

class TestMaker
    attr_reader :spec_root

    def initialize(spec_root)
        @spec_root = spec_root
    end

    def run
        generate_equal_precedence_tests
    end

    private

    attr_reader :current_filename
    attr_reader :current_file
    attr_reader :current_indent

    def generate_equal_precedence_tests
        # Generate operator
        output_file "Syntax/OperatorPrecedence.yaml" do
            output_field "EqualPrecedence" do
                operators_by_precedence.each do |precedence, operators|
                    output "#"
                    output "# Precedence #{precedence}: #{operators.map { |op| op_output(op) }.join(" ")}"
                    output "#"
                    operators.each_with_index do |op1, op1_index|
                        output "# #{op_output(op1)}"
                        operators.drop(op1_index).each do |op2|
                            generate_equal_precedence_test(op1, op2)
                        end
                        output ""
                    end
                end
            end
        end

        output_file "Syntax/OperatorMissingExpressions.yaml" do
            output_field "OperatorMissingExpressions" do
                operators_by_precedence.each do |precedence, operators|
                    operators.each do |operator|
                        generate_missing_expression_test(operator)
                    end
                end
                output ""
            end
        end
    end

    def generate_equal_precedence_test(op1, op2)
        op1_type = op1.type
        op1_type = :infix if op1.key == :indent
        op2_type = op2.type
        op2_type = :infix if op2.key == :indent

        case "#{op1_type} #{op2_type}"
        when "prefix prefix"
            # !-a
            test prefix(op1, prefix(op2, "a"))
            # -!a
            test prefix(op2, prefix(op1, "a"))

        when "prefix infix"
            # a/!b
            test infix("a", op2, prefix(op1, "b"))
            # !a/b
            if op1.direction == :left
                test infix(prefix(op1, "a"), op2, "b")
            else
                test prefix(op1, infix("a", op2, "b"))
            end

        when "prefix postfix"
            # !a?
            if op1.direction == :left
                test postfix(prefix(op1, "a"), op2)
            else
                test prefix(postfix(op1, "a"), op2)
            end

        when "prefix start_delimiter"
            # !(a)
            test prefix(op1, delimited(op2, "a"))
            # (!a)
            test delimited(op2, prefix(op1, "a"))

        when "infix infix"
            # a/b%c
            # a%b/c
            if op1.direction == :left
                test infix(infix("a", op1, "b"), op2, "c")
                test infix(infix("a", op2, "b"), op1, "c") unless op1 == op2
            else
                test infix("a", op1, infix("b", op2, "c"))
                test infix("a", op2, infix("b", op1, "c")) unless op1 == op2
            end

        when "infix postfix"
            # a/b?
            if op1.direction == :left
                test postfix(infix("a", op1, "b"), op2)
            else
                test infix("a", op1, postfix("b", op2))
            end
            # a?/b
            test infix(postfix("a", op2), op1, "b")

        when "infix start_delimiter"
            # (a)/b
            test infix(delimited(op2, "a"), op1, "b")
            # a/(b)
            test infix("a", op1, delimited(op2, "b"))
            # (a/b)
            test delimited(op2, infix("a", op1, "b"))

        when "postfix postfix"
            # a+?
            test postfix(postfix("a", op1), op2)
            # a?+
            test postfix(postfix("a", op2), op1) unless op1 == op2

        when "postfix start_delimiter"
            # (a)?
            test postfix(op1, delimited(op2, "a"))
            # (a?)
            test delimited(op2, postfix("a", op1))

        when "start_delimiter start_delimiter"
            # ({a})
            test delimited(op1, delimited(op2, "a"))
            # {(a)}
            test delimited(op2, delimited(op1, "a")) unless op1 == op2

        else
            # Switch the operators around if they weren't the right direction this time
            generate_equal_precedence_test(op2, op1)
        end
    end

    def generate_missing_expression_test(op)
        output ""
        case op.type
        when :prefix
            error_test prefix(op, ""), "Missing a value on the right side of \"#{op_source(op)}\". Perhaps you closed the file earlier than intended, or didn't mean to put the \"#{op_source(op)}\" there at all?"

        when :infix
            error_test infix("a", op, ""), "Missing a value on the right side of \"#{op_source(op)}\". Perhaps you closed the file earlier than intended, or didn't mean to put the \"#{op_source(op)}\" there at all?"
            error_test infix("", op, "b"), "Missing a value on the left side of \"#{op_source(op)}\". Did you mean for the \"#{op_source(op)}\" to be there?"
            error_test infix("", op, ""), "Missing a value on the left side of \"#{op_source(op)}\". Did you mean for the \"#{op_source(op)}\" to be there?"

        when :postfix
            error_test postfix("", op), "Missing a value on the left side of \"#{op_source(op)}\". Did you mean for the \"#{op_source(op)}\" to be there?"

        when :start_delimiter
            if op.key == :indent
                test infix("a", op, "")
                error_test infix("", op, "b"), "Missing a value on the left side of \"#{op_source(colon)}\". Did you mean for the \"#{op_source(colon)}\" to be there?"
                error_test infix("", op, ""), "Missing a value on the left side of \"#{op_source(colon)}\". Did you mean for the \"#{op_source(colon)}\" to be there?"
            else
                test delimited(op, "")
            end

        else
            raise "Unknown type #{op_type}!"
        end
    end

    def bare(value)
        return value if value.is_a?(Hash)
        if value == ""
            type = "EmptyExpression"
        else
            type = "Bareword"
        end
        { source: value, type: type, expected: value }
    end

    def prefix(op, right)
        right = bare(right)
        {
            type: "PrefixOperation",
            source: "#{op_source(op)}#{right[:source]}",
            expected: {
                "Operator" => op_source(op),
                "Right -> #{right[:type]}" => right[:expected],
            },
        }
    end

    def postfix(left, op)
        left = bare(left)
        {
            type: "PostfixOperation",
            source: "#{left[:source]}#{op_source(op)}",
            expected: {
                "Left -> #{left[:type]}" => left[:expected],
                "Operator" => op_source(op)
            },
        }
    end

    def colon
        BergLang::Parser::OperatorList.berg_operators[":"][:infix]
    end

    def infix(left, op, right)
        left = bare(left)
        right = bare(right)
        if op.key == :indent
            right = delimited(op, right)
            op = colon
        end

        {
            type: "InfixOperation",
            source: "#{left[:source]}#{op_source(op)}#{right[:source]}",
            expected: {
                "Left -> #{left[:type]}" => left[:expected],
                "Operator" => op_source(op),
                "Right -> #{right[:type]}" => right[:expected]
            },
        }
    end

    def fixup_precedence_test(value)
        case value[:source]
        when "(a:\n  b)"
            value[:expected]["Expression -> InfixOperation"]["Right -> DelimitedOperation"]["EndDelimiter"] = ")"
        when "{a:\n  b}"
            value[:expected]["Expression -> InfixOperation"]["Right -> DelimitedOperation"]["EndDelimiter"] = "}"
        when "a:\n  b:\n    c"
            value[:expected]["Right -> DelimitedOperation"]["Expression -> InfixOperation"]["Right -> DelimitedOperation"]["$Space"] = "\n    "
        when "a:\n"
            value[:expected]["Right -> DelimitedOperation"]["Expression -> EmptyExpression"] = "\n"
            value[:expected]["Right -> DelimitedOperation"].delete("$Space")
        else
            value
        end
    end

    def bad_error_test?(value, error)
        outer = value[:expected]
        if value[:type] == "PostfixOperation" && ["--", "++", "+"].include?(outer["Operator"])
            true
        elsif value[:type] == "InfixOperation" && ["*", "+", ":", " ", "\n"].include?(outer["Operator"]) && error =~ /Missing a value on the right side/
            true
        elsif value[:type] == "InfixOperation" && ["+", "-", " ", "\n"].include?(outer["Operator"]) && error =~ /Missing a value on the left side/
            true
        end
    end

    def bad_combination?(value)
        outer = value[:expected]
        if value[:type] == "PostfixOperation" && inner = outer["Left -> PostfixOperation"]
            case "a #{outer["Operator"]} #{inner["Operator"]}"
            when "a ? ?", "a + +", "a + ++"
                true
            end
        elsif value[:type] == "PrefixOperation" && inner = outer["Right -> PrefixOperation"]
            case "#{outer["Operator"]} #{inner["Operator"]} a"
            when "+ + a", "+ ++ a", "- - a", "- -- a"
                true
            end
        end
    end

    def delimited(op, expression)
        expression = bare(expression)
        if op.key == :indent
            indented_source = expression[:source].lines.map { |line| "  #{line}" }.join("")
            source = "\n#{indented_source}"
        else
            source = "#{op_source(op)}#{expression[:source]}#{op_source(op.ended_by)}"
        end
        result = {
            type: "DelimitedOperation",
            source: source,
            expected: {
                "StartDelimiter" => op_source(op),
                "$Space" => (op.key == :indent ? "\n  " : ""),
                "Expression -> #{expression[:type]}" => expression[:expected],
                "EndDelimiter" => op_source(op.ended_by),
            },
        }
    end

    def test(test)
        test = bare(test)
        return if bad_combination?(test)
        old_indent = current_indent
        begin
            fixup_precedence_test(test)
            output "- Berg: #{escape_yaml(test[:source])}"
            indented do
                output_expected_ast_field "Ast -> #{test[:type]}", test[:expected], source: test[:source], line: 1, column: 1
            end
        ensure
            @current_indent = old_indent
        end
    end

    def error_test(test, error)
        test = bare(test)
        return if bad_error_test?(test, error)
        output "- Berg: #{escape_yaml(test[:source])}"
        indented do
            output_field "Error", error
        end
    end

    def operators_by_precedence
        @operators_by_precedence ||= begin
            operators = BergLang::Parser::OperatorList.berg_operators
            operators
                .flat_map { |operator_name, definitions| definitions.values }
                .reject   { |operator| operator.type == :end_delimiter || operator.key == :sof }
                .sort_by  { |operator| operator.precedence }
                .group_by { |operator| operator.precedence }
        end
    end

    #
    # Output helpers
    #

    def indented
        old_indent = current_indent
        begin
            @current_indent += "  "
            yield
        ensure
            @current_indent = old_indent
        end
    end

    def op_source(op)
        op_key = op.is_a?(BergLang::Parser::OperatorDefinition) ? op.key : op

        case op_key
        when :call
            " "
        when :indent
            ""
        when :undent
            ""
        else
            op_key
        end
    end

    def output_expected_ast_field(name, value, source:, line:, column:)
        if value.is_a?(Hash)
            output "#{name}:"
            indented do
                value.each do |name, value|
                    line, column = output_expected_ast_field(name, value, source: source, line: line, column: column)
                end
            end
        else
            begin_line, begin_column = [ line, column ]
            line, column = add_to_location(begin_line, begin_column, value)
            if source.scan(value).size > 1
                range = "#{begin_line}@#{begin_column}"
                if value.size == 0
                    range << "+0"
                else
                    end_line, end_column = add_to_location(begin_line, begin_column, value[0..-2])
                    if begin_line != end_line
                        range << "-#{end_line}@#{end_column}"
                    elsif begin_column != end_column
                        range << "-#{end_column}"
                    end
                end
                value = "#{range} = #{value}"
            end
            output "#{name}: #{escape_yaml(value)}" unless name.start_with?("$Space")
        end
        [ line, column ]
    end

    #
    # Move the line/column indicator forward as if the string was appended.
    #
    def add_to_location(line, column, string)
        string = string[:source] if string.is_a?(Hash)
        lines = string.lines
        lines << "" if lines.empty? || lines[-1][-1] == "\n" || lines.empty?

        line += lines.size - 1
        column = 1 if lines.size > 1
        column += lines[-1].size
        [ line, column ]
    end

    def output_field(name, value=nil, &block)
        if value.is_a?(Hash)
            output "#{name}:"
            indented do
                value.each do |name, value|
                    output_field name, value
                end
            end
        elsif block
            output "#{name}:"
            indented do
                block.call
            end
        else
            output "#{name}: #{escape_yaml(value)}"
        end
    end

    def output_file(filename, &block)
        filename = File.join(spec_root, filename)
        raise "Cannot open #{filename} while #{current_filename} is already open!" if current_filename
        puts "Writing #{filename} ..."
        File.open(filename, "w") do |file|
            @current_filename = filename
            @current_file = file
            @current_indent = ""
            block.call
        end
        @current_filename = nil
        @current_file = nil
    end

    def op_output(op)
        op_string = op.key
        op_string = op_string.inspect if op_string =~ /\s|\n/
        if op.type == :infix
            op_string
        else
            "#{op.type}.#{op_string}"
        end
    end

    def escape_yaml(string)
        case string
        when "", "-", "*", ".", /[\n\t\\"':]/, /\A[%>!?|&*,{}]/, /\s\Z/
            string.inspect
        else
            string
        end
    end

    def output(string)
        if string == ""
            current_file.puts ""
        else
            string.each_line do |line|
                if line == ""
                    current_file.puts ""
                else
                    current_file.puts "#{current_indent}#{line}"
                end
            end
        end
    end
end

spec_root = File.expand_path("../../../spec_tests", __FILE__)
if !File.directory?(spec_root)
    raise "#{spec_root} does not exist!"
end
test_maker = TestMaker.new(spec_root)
test_maker.run
