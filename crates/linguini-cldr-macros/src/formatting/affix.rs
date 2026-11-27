use linguini_cldr::NumberAffixPart;
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;

use super::string_field;

pub(super) type CompiledAffix = Vec<NumberAffixPart<String>>;

pub(super) struct NumberAffixSymbols {
    pub(super) minus: String,
    pub(super) plus: String,
    pub(super) percent: String,
    pub(super) per_mille: String,
}

impl NumberAffixSymbols {
    pub(super) fn extract(symbols: &Value) -> Result<Self, String> {
        Ok(Self {
            minus: string_field(symbols, "minusSign", "number symbols")?,
            plus: string_field(symbols, "plusSign", "number symbols")?,
            percent: string_field(symbols, "percentSign", "number symbols")?,
            per_mille: string_field(symbols, "perMille", "number symbols")?,
        })
    }
}

pub(super) fn compile_affix(
    affix: &str,
    symbols: &NumberAffixSymbols,
) -> Result<(CompiledAffix, u16), String> {
    let mut parts = Vec::new();
    let mut literal = String::with_capacity(affix.len());
    let mut scale = 1;
    let mut quoted = false;
    let mut chars = affix.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\'' {
            match character {
                '-' if !quoted => literal.push_str(&symbols.minus),
                '+' if !quoted => literal.push_str(&symbols.plus),
                '%' | '\u{2030}' if !quoted => {
                    if scale != 1 {
                        return Err(format!(
                            "number affix has multiple scaling symbols: `{affix}`"
                        ));
                    }
                    scale = if character == '%' { 100 } else { 1000 };
                    literal.push_str(if character == '%' {
                        &symbols.percent
                    } else {
                        &symbols.per_mille
                    });
                }
                '¤' if !quoted => {
                    let mut width = 1;
                    while chars.peek() == Some(&'¤') {
                        chars.next();
                        width += 1;
                    }
                    if width != 1 {
                        return Err(format!(
                            "unsupported currency marker width {width} in affix `{affix}`"
                        ));
                    }
                    if !literal.is_empty() {
                        parts.push(NumberAffixPart::Literal(std::mem::take(&mut literal)));
                    }
                    parts.push(NumberAffixPart::CurrencySymbol);
                }
                '#' | '@' | '0'..='9' | '.' | ',' | '*' | ';' if !quoted => {
                    return Err(format!(
                        "unquoted number-pattern character `{character}` in affix `{affix}`"
                    ));
                }
                _ => literal.push(character),
            }
            continue;
        }
        if chars.peek() == Some(&'\'') {
            chars.next();
            literal.push('\'');
        } else {
            quoted = !quoted;
        }
    }
    if quoted {
        return Err(format!("unterminated quote in number affix `{affix}`"));
    }
    if !literal.is_empty() {
        parts.push(NumberAffixPart::Literal(literal));
    }
    Ok((parts, scale))
}

pub(super) fn affix_tokens(parts: &[NumberAffixPart<String>]) -> TokenStream {
    match parts {
        [] => quote! { NumberAffix::Literal("") },
        [NumberAffixPart::Literal(text)] => quote! { NumberAffix::Literal(#text) },
        [NumberAffixPart::CurrencySymbol] => quote! { NumberAffix::CurrencySymbol },
        parts => {
            let parts = parts.iter().map(|part| match part {
                NumberAffixPart::Literal(text) => quote! { NumberAffixPart::Literal(#text) },
                NumberAffixPart::CurrencySymbol => quote! { NumberAffixPart::CurrencySymbol },
            });
            quote! { NumberAffix::Parts(&[#(#parts),*]) }
        }
    }
}
