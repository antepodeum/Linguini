// Maintainer oracle: compile with pkg-config --cflags --libs icu-i18n icu-uc.
// Input is pattern<TAB>exact decimal input. Output is one formatted line per input.
// An optional three-letter currency code isolates affixes at integer precision,
// with legacy currency-spacing insertion disabled.
#include <unicode/decimfmt.h>
#include <unicode/dcfmtsym.h>
#include <unicode/locid.h>
#include <unicode/stringpiece.h>
#include <unicode/unistr.h>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
    if (argc > 2 || (argc == 2 && std::string(argv[1]).size() != 3)) return 2;
    std::string line;
    while (std::getline(std::cin, line)) {
        const auto separator = line.find('\t');
        if (separator == std::string::npos) return 2;
        const auto pattern = line.substr(0, separator);
        const auto input = line.substr(separator + 1);
        UErrorCode status = U_ZERO_ERROR;
        auto* symbols = new icu::DecimalFormatSymbols(icu::Locale::getUS(), status);
        if (argc == 2) {
            for (UBool before : {false, true}) {
                symbols->setPatternForCurrencySpacing(UNUM_CURRENCY_MATCH, before,
                    icu::UnicodeString::fromUTF8("[]"));
            }
        }
        icu::DecimalFormat formatter(icu::UnicodeString::fromUTF8(pattern), symbols, status);
        if (argc == 2) {
            const UChar currency[] = {static_cast<UChar>(argv[1][0]),
                static_cast<UChar>(argv[1][1]), static_cast<UChar>(argv[1][2]), 0};
            formatter.setCurrency(currency, status);
            formatter.setMinimumFractionDigits(0);
            formatter.setMaximumFractionDigits(0);
        }
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
