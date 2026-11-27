use linguini_cldr::{NumberAffix, NumberAffixPart};

pub struct CurrencyAffixCase {
    pub name: &'static str,
    pub pattern: &'static str,
    pub prefix: NumberAffix,
    pub suffix: NumberAffix,
    pub negative: Option<(NumberAffix, NumberAffix)>,
    pub positive: &'static str,
    pub negative_text: &'static str,
}

pub const CASES: &[CurrencyAffixCase] = &[
    CurrencyAffixCase {
        name: "literal_prefix",
        pattern: "'¤'0",
        prefix: NumberAffix::Literal("¤"),
        suffix: NumberAffix::Literal(""),
        negative: None,
        positive: "¤1",
        negative_text: "-¤1",
    },
    CurrencyAffixCase {
        name: "symbol_prefix",
        pattern: "¤0",
        prefix: NumberAffix::CurrencySymbol,
        suffix: NumberAffix::Literal(""),
        negative: None,
        positive: "$1",
        negative_text: "-$1",
    },
    CurrencyAffixCase {
        name: "literal_then_symbol",
        pattern: "'¤'¤0",
        prefix: NumberAffix::Parts(&[
            NumberAffixPart::Literal("¤"),
            NumberAffixPart::CurrencySymbol,
        ]),
        suffix: NumberAffix::Literal(""),
        negative: None,
        positive: "¤$1",
        negative_text: "-¤$1",
    },
    CurrencyAffixCase {
        name: "symbol_then_literal",
        pattern: "¤'¤'0",
        prefix: NumberAffix::Parts(&[
            NumberAffixPart::CurrencySymbol,
            NumberAffixPart::Literal("¤"),
        ]),
        suffix: NumberAffix::Literal(""),
        negative: None,
        positive: "$¤1",
        negative_text: "-$¤1",
    },
    CurrencyAffixCase {
        name: "literal_suffix",
        pattern: "¤0'¤'",
        prefix: NumberAffix::CurrencySymbol,
        suffix: NumberAffix::Literal("¤"),
        negative: None,
        positive: "$1¤",
        negative_text: "-$1¤",
    },
    CurrencyAffixCase {
        name: "mixed_suffix",
        pattern: "0¤'¤'",
        prefix: NumberAffix::Literal(""),
        suffix: NumberAffix::Parts(&[
            NumberAffixPart::CurrencySymbol,
            NumberAffixPart::Literal("¤"),
        ]),
        negative: None,
        positive: "1$¤",
        negative_text: "-1$¤",
    },
    CurrencyAffixCase {
        name: "repeated_slots",
        pattern: "¤'¤'¤0",
        prefix: NumberAffix::Parts(&[
            NumberAffixPart::CurrencySymbol,
            NumberAffixPart::Literal("¤"),
            NumberAffixPart::CurrencySymbol,
        ]),
        suffix: NumberAffix::Literal(""),
        negative: None,
        positive: "$¤$1",
        negative_text: "-$¤$1",
    },
    CurrencyAffixCase {
        name: "quoted_apostrophe",
        pattern: "'a''¤'¤0",
        prefix: NumberAffix::Parts(&[
            NumberAffixPart::Literal("a'¤"),
            NumberAffixPart::CurrencySymbol,
        ]),
        suffix: NumberAffix::Literal(""),
        negative: None,
        positive: "a'¤$1",
        negative_text: "-a'¤$1",
    },
    CurrencyAffixCase {
        name: "negative_affix",
        pattern: "'¤'¤0;('¤'¤0)",
        prefix: NumberAffix::Parts(&[
            NumberAffixPart::Literal("¤"),
            NumberAffixPart::CurrencySymbol,
        ]),
        suffix: NumberAffix::Literal(""),
        negative: Some((
            NumberAffix::Parts(&[
                NumberAffixPart::Literal("(¤"),
                NumberAffixPart::CurrencySymbol,
            ]),
            NumberAffix::Literal(")"),
        )),
        positive: "¤$1",
        negative_text: "(¤$1)",
    },
    CurrencyAffixCase {
        name: "literal_dollar",
        pattern: "'$&¤'0",
        prefix: NumberAffix::Literal("$&¤"),
        suffix: NumberAffix::Literal(""),
        negative: None,
        positive: "$&¤1",
        negative_text: "-$&¤1",
    },
];
