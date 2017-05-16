#!ruby
require "bundler/setup"
require "berg_lang/parser/operator_list"

class TestMaker
    attr_reader :spec_root

    def initialize(spec_root)
        @spec_root = spec_root
    end

    def run
        generate_precedence_tests
    end

    private

    attr_reader :current_filename
    attr_reader :current_file
    attr_reader :current_indent

    def generate_precedence_tests
        # Generate operator
        output_file "Syntax/OperatorPrecedence.yaml" do
            operators_by_precedence.each do |precedence1, operators1|
                indented do
                    output "#"
                    output "# Precedence #{precedence1}: #{operators1.map { |op| op_output(op) }.join(" ")}"
                    output "#"
                    operators_by_precedence.each do |precedence2, operators2|
                        # Only process each pair of operators once.
                        next if precedence2 < precedence1
                        indented do
                            output "#"
                            output "# Against precedence #{precedence2}: #{operators2.map { |op| op_output(op) }.join(" ")}"
                            output "#"
                            operators1.each_with_index do |op1, op1_index|
                                output "#"
                                output "# #{op_output(op1)}"
                                output "#"
                                if precedence1 == precedence2
                                    operators2.drop(op1_index).each do |op2|
                                        generate_precedence_test(op1, op2)
                                    end
                                else
                                    operators2.each do |op2|
                                        generate_precedence_test(op1, op2)
                                    end
                                end
                                output ""
                            end
                        end
                        output ""
                    end
                end
                output ""
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

    def generate_precedence_test(op1, op2)
        if op1.key == :indent
            op1_indent = true
            op1 = colon
        end
        if op2.key == :indent
            op2_indent = true
            op2 = colon
        end
        if op1.precedence == op2.precedence
            winner = op1.direction
        elsif op1.precedence < op2.precedence || op1.declaration?
            winner = :op1
        else
            winner = :op2
        end

        case "#{op1.type} #{op2.type}"
        when "prefix prefix"
            # !-a
            test prefix(op1, prefix(op2, "a"))
            # -!a
            test prefix(op2, prefix(op1, "a"))

        when "prefix infix"
            # a/!b, a:\n  !b
            test infix("a", op2, prefix(op1, "b"))

            # !a/b, !a:\n  b
            if [ :op1, :left ].include?(winner)
                test infix(prefix(op1, "a"), op2, "b")
            else
                test prefix(op1, infix("a", op2, "b"))
            end

        when "prefix postfix"
            # !a?
            if [ :op1, :left ].include?(winner)
                test postfix(prefix(op1, "a"), op2)
            else
                test prefix(op1, postfix("a", op2))
            end

        when "prefix open"
            # !(a)
            test prefix(op1, delimited(op2, "a"))
            # (!a)
            test delimited(op2, prefix(op1, "a"))

        when "infix infix"
            # a/b%c, a:\n  b%c, a:\n  b:\n    c
            if [ :op1, :left ].include?(winner)
                test infix(infix("a", op1, "b"), op2, "c")
            else
                test infix("a", op1, infix("b", op2, "c"))
            end
            # a%b/c, a%b:\n  c
            if op1 != op2
                if [ :op2, :left ].include?(winner)
                    test infix(infix("a", op2, "b"), op1, "c")
                else
                    test infix("a", op2, infix("b", op1, "c"))
                end
            end

        when "infix postfix"
            # a/b?, a\n  b?
            if [ :op1, :left ].include?(winner)
                test postfix(infix("a", op1, "b"), op2)
            else
                test infix("a", op1, postfix("b", op2))
            end
            # a?/b, a?:\n  b
            test infix(postfix("a", op2), op1, "b")

        when "infix open"
            # (a)/b, (a):\n  b
            test infix(delimited(op2, "a"), op1, "b")
            # a/(b), a:\n  (b)
            test infix("a", op1, delimited(op2, "b"))
            # (a/b), (a:\n  b)
            test delimited(op2, infix("a", op1, "b"))

        when "postfix postfix"
            # a+?
            test postfix(postfix("a", op1), op2)
            # a?+
            test postfix(postfix("a", op2), op1) unless op1 == op2

        when "postfix open"
            # (a)?
            test postfix(delimited(op2, "a"), op1)
            # (a?)
            test delimited(op2, postfix("a", op1))

        when "open open"
            # ({a})
            test delimited(op1, delimited(op2, "a"))
            # {(a)}
            test delimited(op2, delimited(op1, "a")) unless op1 == op2

        else
            # Switch the operators around if they weren't the right direction this time
            generate_precedence_test(op2, op1)
        end
    end

    def generate_missing_expression_test(op)
        output ""
        case op.type
        when :prefix
            error_test prefix(op, ""), "No value after \"#{op_source(op)}\"! Did you mean to put a value or variable there?"

        when :infix
            error_test infix("a", op, ""), "No value after \"#{op_source(op)}\"! Did you mean to put a value or variable there?"
            error_test infix("", op, "b"), "No value before \"#{op_source(op)}\"! Did you mean to put a value or variable there?"
            error_test infix("", op, ""), "No value before \"#{op_source(op)}\"! Did you mean to put a value or variable there?"

        when :postfix
            error_test postfix("", op), "No value before \"#{op_source(op)}\"! Did you mean to put a value or variable there?"

        when :open
            if op.key == :indent
                test infix("a", op, "")
                error_test infix("", op, "b"), "No value before \":\"! Did you mean to put a value or variable there?"
                error_test infix("", op, ""), "No value before \":\"! Did you mean to put a value or variable there?"
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
        is_indent = (op.key == :indent)
        if is_indent
            right = delimited(op, right)
            op = colon
        end

        result = {
            type: "InfixOperation",
            source: "#{left[:source]} #{op_source(op)} #{right[:source]}",
            expected: {
                "Left -> #{left[:type]}" => left[:expected],
                "$SpaceLeft" => " ",
                "Operator" => op_source(op),
                "$SpaceRight" => " ",
                "Right -> #{right[:type]}" => right[:expected],
            },
        }
        if op.key == :call
            result[:expected]["$SpaceLeft"] = ""
            result[:expected]["Operator"] = ""
            result[:expected]["$SpaceRight"] = "   "
        elsif is_indent
            result[:expected]["$SpaceRight"] = ""
            result[:expected]["$Space"] = " #{result[:expected]["$Space"]}"
        end
        result
    end

    def fixup_precedence_test(value)
        expected = value[:expected]
        operator = expected["Operator"]
        type = value[:type]

        return nil if operator == ":" && type == "PrefixOperation"

        case value[:source]
        when "(a:\n  b)"
            expected["Expression -> InfixOperation"]["Right -> DelimitedOperation"]["Close"] = ")"

        when "{a:\n  b}"
            expected["Expression -> InfixOperation"]["Right -> DelimitedOperation"]["Close"] = "}"

        when "a:\n  b:\n    c"
            expected["Right -> DelimitedOperation"]["Expression -> InfixOperation"]["Right -> DelimitedOperation"]["$Space"] = "\n    "

        when "a : \n"
            expected.delete("$SpaceRight")
            expected["Right -> DelimitedOperation"] = {
                "Expression -> EmptyExpression" => "",
            }

        when "++a", "--a", "+++a", "---a"
            right_operator = expected["Right -> PrefixOperation"]["Operator"]
            if [ "+", "-" ].include?(operator)
                value[:source] = "#{operator} #{right_operator}a"
                value[:expected] = {
                    "Operator" => operator,
                    "$Space" => " ",
                    "Right -> PrefixOperation" => expected["Right -> PrefixOperation"],
                }
            end

        when "a++", "a--", "a+++", "a??"
            left_operator = expected["Left -> PostfixOperation"]["Operator"]
            if [ "+", "-", "?" ].include?(left_operator)
                value[:source] = "a#{left_operator} #{operator}"
                value[:expected] = {
                    "Left -> PostfixOperation" => expected["Left -> PostfixOperation"],
                    "$Space" => " ",
                    "Operator" => operator,
                }
            end

        when /\Aa[,;] [-+] b\Z/
            left_operator = expected["Left -> PostfixOperation"]["Operator"]
            value[:expected] = {
                "Left -> Bareword" => "a",
                "Operator" => left_operator,
                "$Space" => " ",
                "Right -> PrefixOperation" => {
                    "Operator" => operator,
                    "$Space" => " ",
                    "Right -> Bareword" => "b"
                }
            }

        when /\Aa[,;] ([ \n]) b\Z/
            left_operator = expected["Left -> PostfixOperation"]["Operator"]
            value[:expected] = {
                "Left -> Bareword" => "a",
                "Operator" => left_operator,
                "$Space" => " #{$1} ",
                "Right -> Bareword" => "b",
            }

        end

        value
    end

    def bad_error_test?(value, error)
        case value[:source]
        # Infix operators that are also postfix
        when "a ; ", "a , ", "a * ", "a + ", "a : ", "a   ", "a \n "
            true
        # Infix operators that are also prefix
        when " + b", " - b", "   b", " \n b", " : b", " : ", " - ", " + ", "   ", " \n "
            true
        # Postfix operators that are also prefix
        when "++", "--", "+"
            value[:type] == "PostfixOperation"
        # TODO this should be an error, but right now isn't.'
        when ":"
            true
        end
    end

    def delimited(op, expression)
        expression = bare(expression)
        if op.key == :indent
            indented_source = expression[:source].lines.map { |line| "  #{line}" }.join("")
            source = "\n#{indented_source}"
        else
            source = "#{op_source(op)}#{expression[:source]}#{op_source(op.closed_by)}"
        end
        result = {
            type: "DelimitedOperation",
            source: source,
            expected: {
                "Open" => op_source(op),
                "$Space" => (op.key == :indent ? "\n  " : ""),
                "Expression -> #{expression[:type]}" => expression[:expected],
                "Close" => op_source(op.closed_by),
            },
        }
    end

    def test(test)
        test = bare(test)
        test = fixup_precedence_test(test)
        return false if test.nil?
        old_indent = current_indent
        begin
            output "- Berg: #{escape_yaml(test[:source])}"
            indented do
                output_expected_ast_field "Ast -> #{test[:type]}", test[:expected], source: test[:source], line: 1, column: 1
            end
        ensure
            @current_indent = old_indent
        end
        true
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
                .reject   { |operator| operator.type == :close || operator.key == :sof }
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
        when "", "-", "*", ".", /[\n\t\\"':]/, /\A[%>!?|&*,{}]/, /\s\Z/, /\A\s/, /\A-\s/
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
