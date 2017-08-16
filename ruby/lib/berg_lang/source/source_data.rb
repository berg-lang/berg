require_relative "source_range"

module BergLang
    module Source
        class SourceData
            attr_reader :source
            attr_reader :read_timestamp
            attr_accessor :size
            attr_accessor :checksum
            attr_reader :line_start_codepoint_indices

            def initialize(source, read_timestamp=DateTime.now)
                @source = source
                @read_timestamp = read_timestamp
            end

            def codepoint_index_for_line(line)
                line_start_codepoint_indices[line-1]
            end

            def line_for_codepoint_index(codepoint_index)
                low = 0
                high = lines.size - 1
                while low <= high
                    mid = (high - low) / 2
                    case codepoint_index <=> line_start_codepoint_indexes[mid]
                    when 0
                        return mid + 1
                    when 1
                        low = mid + 1
                    when -1
                        high = mid - 1
                    else
                        raise "Unexpected result #{line_start_codepoint_indexes[mid] <=> codepoint_index}"
                    end
                end
                low + 1
            end

            def column_for_codepoint_index(line, codepoint_index)
                codepoint_index - codepoint_index_for_line(line) + 1
            end

            def source_range(codepoint_index_range)
                SourceRange.new(self, codepoint_index_range)
            end
        end
    end
end