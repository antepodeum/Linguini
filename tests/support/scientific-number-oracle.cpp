// Maintainer oracle: compile with pkg-config --cflags --libs icu-i18n icu-uc.
// Input is pattern<TAB>exact decimal input. Output is one formatted line per input.
#include <unicode/decimfmt.h>
#include <unicode/dcfmtsym.h>
#include <unicode/locid.h>
#include <unicode/stringpiece.h>
#include <unicode/unistr.h>
#include <iostream>
#include <string>

int main() {
    std::string line;
    while (std::getline(std::cin, line)) {
        const auto separator = line.find('\t');
        if (separator == std::string::npos) return 2;
        const auto pattern = line.substr(0, separator);
        const auto input = line.substr(separator + 1);
        UErrorCode status = U_ZERO_ERROR;
        auto* symbols = new icu::DecimalFormatSymbols(icu::Locale::getUS(), status);
        icu::DecimalFormat formatter(icu::UnicodeString::fromUTF8(pattern), symbols, status);
        formatter.setRoundingMode(icu::DecimalFormat::kRoundHalfUp);
        icu::UnicodeString result;
        formatter.format(icu::StringPiece(input), result, nullptr, status);
        if (U_FAILURE(status)) {
            std::cerr << u_errorName(status) << '\n';
            return 3;
        }
        std::string output;
        result.toUTF8String(output);
        std::cout << output << '\n';
    }
}
