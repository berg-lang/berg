require "set"

class Variants
    attr_reader :expression
    attr_reader :infix
    attr_reader :prefix
    attr_reader :postfix

    def initialize(expression, infix, prefix, postfix)
        @expression = expression
        @infix = infix
        @prefix = prefix
        @postfix = postfix
    end

    def variants
        variants = []
        variants << "#{expression == :insignificant ? "insignificant " : ""}expression" if expression
        variants << "#{infix == :insignificant ? "insignificant " : ""}infix" if infix
        variants << "#{prefix == :insignificant ? "insignificant " : ""}prefix" if prefix
        variants << "#{postfix == :insignificant ? "insignificant " : ""}postfix" if postfix
        variants
    end

    def to_s
        variants.join(", ")
    end

    def raw_results
        {
            "operand"           => result(:operand, choose_expression.merge(choose_prefix)),
            "operator"          => result(:operator, choose_postfix.merge(choose_infix)),
            "operand+leading"   => result(:operand, choose_expression.merge(choose_prefix)),
            "operator+leading"  => result(:operator, choose_expression.merge(choose_prefix)),
            "operand+trailing"  => result(:operand, choose_expression),
            "operator+trailing" => result(:operator, choose_postfix)
        }
    end

    def unique_results(results)
        results.delete("operand+trailing") if results["operand+trailing"] == results["operand"]
        results.delete("operator+trailing") if results["operator+trailing"] == results["operator"]
        if results["operand+trailing"] == results["operator+trailing"] && results["operand+trailing"]
            results["trailing"] = results["operand+trailing"]
            results.delete("operand+trailing")
            results.delete("operator+trailing")
        end

        results.delete("operand+leading") if results["operand+leading"] == results["operand"]
        results.delete("operator+leading") if results["operator+leading"] == results["operator"]
        if results["operand+leading"] == results["operator+leading"] && results["operand+leading"]
            results["leading"] = results["operand+leading"]
            results.delete("operand+leading")
            results.delete("operator+leading")
        end

        results.group_by { |title, result| result }.map do |result, group|
            [ result, group.map { |title, result| title }.join(", ") ]
        end.to_h
    end

    def result(preference, results)
        case results.keys.sort.join(" ")
        when "expression postfix"
            if preference == :operator
                results.delete(:expression)
            else
                results.delete(:postfix)
            end
        when "infix prefix"
            if preference == :operator
                results.delete(:prefix)
            else
                results.delete(:infix)
            end
        end

        case results.keys.sort.join(" ")
        when "expression"
            append = true
            resolve_as = :operator
            next_preference = :operator
        when "postfix"
            append = true
            resolve_as = :operand
            next_preference = :operator
        when "infix"
            append = true
            resolve_as = :operand
            next_preference = :operand
        when "prefix"
            append = true
            resolve_as = :operator
            next_preference = :operand
        when "expression infix"
            next_preference = (preference == :operand) ? :operator : :operand
        when "postfix prefix"
            next_preference = preference
        when "expression prefix"
            resolve_as = :operator
            next_preference = (preference == :operator || results[:prefix] == :insignificant) ? :operand : :operator
        when "infix postfix"
            resolve_as = :operand
            next_preference = (preference == :operand || results[:postfix] == :insignificant) ? :operator : :operand
        else
            raise "Unexpected combo #{results.keys.sort.join(" ")}!"
        end

        {
            resolve_as: resolve_as,
            append: append,
            next_preference: next_preference,
            tokens: results.keys.join(" and "),
        }
    end

    def result_string(results)
        results.keys.join(" and ")
    end

    def choose_expression
        return { expression: expression } if expression
        return { postfix: postfix } if postfix
        return { prefix: prefix } if prefix
        return { infix: infix } if infix
    end

    def choose_infix
        return { infix: infix } if infix
        return { prefix: prefix } if prefix
        return { postfix: postfix } if postfix
        return { expression: expression } if expression
    end

    def choose_prefix
        return { prefix: prefix } if prefix
        return { infix: infix } if infix
        return { expression: expression } if expression
        return { postfix: postfix } if postfix
    end

    def choose_postfix
        return { postfix: postfix } if postfix
        return { expression: expression } if expression
        return { infix: infix } if infix
        return { prefix: prefix } if prefix
    end
end

all_variants = []
[true, false].each do |expression|
    [true, false].each do |infix|
        [true, false ].each do |prefix|
            [true, false ].each do |postfix|
                sum = (expression ? 1 : 0) + (infix ? 1 : 0) + (prefix ? 1 : 0) + (postfix ? 1 : 0)
                next unless sum > 0 && sum >= 2
                all_variants << Variants.new(expression, infix, prefix, postfix)
            end
        end
    end
end

def group_results(all_variants, title, &block)
    puts "#{title}\n"
    all_variants.group_by do |variants|
        results = variants.raw_results.map { |title,result| [ title, block.call(result) ] }.to_h
        variants.unique_results(results)
    end.each do |results, grouped_variants|
        puts "--------------------------\n"
        results.each do |c, result|
            puts "#{result} -> #{c}\n"
        end
        grouped_variants.each do |variants|
            puts "- #{variants}\n"
        end
        puts ""
    end
end

group_results(all_variants, "next_preference") { |result| result[:next_preference] }
#group_results("resolve") { |result| result[:swap] }
#group_results("append") { |result| result[:append] }
#group_results("swap") { |result| result[:swap] }
#group_results("if_operand") { |result| result[:if_operand] }
#group_results("if_operator") { |result| result[:if_operator] }

# Class A: 3-4 things
# ===============================
#
# ===============================

# - expression, infix, prefix, postfix
#   operand, operator+leading -> expression and prefix
#   operator -> postfix and infix
#   operator+trailing -> postfix

# - expression, infix, prefix
#   operand, operator+leading -> expression and prefix
#   operator -> expression and infix
#   operator+trailing -> expression

# - expression, infix, postfix
#   operand, operator+leading -> expression and infix
#   operator -> postfix and infix
#   operator+trailing -> postfix
#
# - expression, prefix, postfix
#   operand, operator+leading -> expression and prefix
#   operator -> postfix and prefix
#   operator+trailing -> postfix
#
# - infix, prefix, postfix
#   operand, operator+leading -> postfix and prefix
#   operator -> postfix and infix
#   operator+trailing -> postfix
#
# Class B: 2 things with different right sides
#
# - expression, infix
#   operand, operator -> expression and infix
#   operator+trailing -> expression
#
# - expression, prefix
#   operand, operator -> expression and prefix
#   operator+trailing -> expression
#
# - infix, postfix
#   operand, operator -> postfix and infix
#   operator+trailing -> postfix
#
# - prefix, postfix
#   operand, operator -> postfix and prefix
#   operator+trailing -> postfix
#
# Class C: 2 things with same right side
#
# - expression, postfix
#   operand, operator+leading -> expression
#   operator -> postfix
#
# - infix, prefix
#   operand, operator+leading -> prefix
#   operator -> infix
#

# Class D: 1 thing

