module BergLang
    module Parser
        class Context
            attr_reader :output
            attr_reader :syntax_tree

            def initialize(output, source)
                @output = output
                @syntax_tree = SyntaxTree.new(source)
            end

            def source
                syntax_tree.source
            end
        end
    end
end
