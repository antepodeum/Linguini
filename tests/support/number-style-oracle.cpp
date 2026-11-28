// Maintainer oracle: compile with pkg-config --cflags --libs icu-i18n icu-uc.
// Input: locale<TAB>pinned number pattern<TAB>minimum grouping digits<TAB>exact input.
#include <unicode/decimfmt.h>
#include <unicode/dcfmtsym.h>
#include <unicode/fpositer.h>
#include <unicode/locid.h>
#include <unicode/stringpiece.h>
#include <unicode/unistr.h>
#include <unicode/unum.h>
#include <array>
#include <iostream>
#include <string>
#include <vector>

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
        std::array<icu::UnicodeString, 10> digits;
        auto digitSymbol = [](int digit) {
            return digit == 0 ? icu::DecimalFormatSymbols::kZeroDigitSymbol
                : static_cast<icu::DecimalFormatSymbols::ENumberFormatSymbol>(
                    icu::DecimalFormatSymbols::kOneDigitSymbol + digit - 1);
        };
        for (int digit = 0; digit < 10; digit++) digits[digit] = symbols->getSymbol(digitSymbol(digit));
        // ICU 78.3 truncates supplementary digits in multi-digit exponents. Keep its
        // arithmetic and field boundaries, then localize only numeric fields by full strings.
        for (int digit = 0; digit < 10; digit++) {
            symbols->setSymbol(digitSymbol(digit), icu::UnicodeString(static_cast<UChar>('0' + digit)));
        }
        icu::DecimalFormat formatter(icu::UnicodeString::fromUTF8(fields[1]), symbols, status);
        formatter.setMinimumGroupingDigits(std::stoi(fields[2]));
        formatter.setRoundingMode(icu::DecimalFormat::kRoundHalfUp);
        icu::UnicodeString result;
        icu::FieldPositionIterator positions;
        formatter.format(icu::StringPiece(fields[3]), result, &positions, status);
        if (U_FAILURE(status)) {
            std::cerr << u_errorName(status) << '\n';
            return 3;
        }
        std::vector<bool> numeric(result.length());
        icu::FieldPosition position;
        while (positions.next(position)) {
            if (position.getField() != UNUM_INTEGER_FIELD
                && position.getField() != UNUM_FRACTION_FIELD
                && position.getField() != UNUM_EXPONENT_FIELD) continue;
            for (int32_t index = position.getBeginIndex(); index < position.getEndIndex(); index++) numeric[index] = true;
        }
        icu::UnicodeString localized;
        for (int32_t index = 0; index < result.length();) {
            const auto character = result.char32At(index);
            if (numeric[index] && character >= '0' && character <= '9') localized.append(digits[character - '0']);
            else localized.append(character);
            index += U16_LENGTH(character);
        }
        std::string output;
        localized.toUTF8String(output);
        std::cout << output << '\n';
    }
}
