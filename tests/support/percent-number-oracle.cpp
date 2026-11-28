// Maintainer oracle: compile with pkg-config --cflags --libs icu-i18n icu-uc.
// Input: locale<TAB>pinned percent pattern<TAB>minimum grouping digits<TAB>exact input.
#include <unicode/decimfmt.h>
#include <unicode/dcfmtsym.h>
#include <unicode/locid.h>
#include <unicode/stringpiece.h>
#include <unicode/unistr.h>
#include <array>
#include <iostream>
#include <string>

int main() {
    std::string line;
    while (std::getline(std::cin, line)) {
        std::array<std::string, 4> fields;
        std::size_t start = 0;
        for (std::size_t index = 0; index < 3; index++) {
            const auto end = line.find('\t', start);
            if (end == std::string::npos) return 2;
            fields[index] = line.substr(start, end - start);
            start = end + 1;
        }
        fields[3] = line.substr(start);
        UErrorCode status = U_ZERO_ERROR;
        const auto locale = icu::Locale::forLanguageTag(fields[0], status);
        auto* symbols = new icu::DecimalFormatSymbols(locale, status);
        icu::DecimalFormat formatter(icu::UnicodeString::fromUTF8(fields[1]), symbols, status);
        formatter.setMinimumGroupingDigits(std::stoi(fields[2]));
        formatter.setRoundingMode(icu::DecimalFormat::kRoundHalfUp);
        icu::UnicodeString result;
        formatter.format(icu::StringPiece(fields[3]), result, nullptr, status);
        if (U_FAILURE(status)) {
            std::cerr << u_errorName(status) << '\n';
            return 3;
        }
        std::string output;
        result.toUTF8String(output);
        std::cout << output << '\n';
    }
}
