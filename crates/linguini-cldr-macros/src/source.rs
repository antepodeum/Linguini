use crate::formatting::{
    generate_formatting_tables, generate_text_direction_table, FormattingCoverage,
};
use crate::source_paths::CldrSource;
use linguini_cldr::{
    parse_plural_rule, Condition, Operand, OperandExpression, PluralRule, Range, RangeList,
    Relation, RelationOperator,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde_json::Value;
use std::fs;

pub(crate) struct GeneratedTables {
    pub(crate) tokens: TokenStream,
    pub(crate) plural_locales: usize,
    pub(crate) plural_categories: usize,
    pub(crate) formatting: FormattingCoverage,
}

pub(crate) fn generate_compiled_tables(source: &CldrSource) -> Result<GeneratedTables, String> {
    let plural_source = fs::read_to_string(source.plurals())
        .map_err(|error| format!("{}: {error}", source.plurals().display()))?;
    let (plural_tables, plural_locales, plural_categories) =
        generate_plural_tables(&plural_source)?;
    let (direction_table, text_direction_locales, text_direction_exclusions) =
        generate_text_direction_table(source.layout_main(), source.locales())?;
    let (formatting_tables, mut formatting) =
        generate_formatting_tables(source.numbers_main(), source.dates_main(), source.locales())?;
    formatting.text_direction_locales = text_direction_locales;
    formatting.text_direction_exclusions = text_direction_exclusions;

    Ok(GeneratedTables {
        tokens: quote! {
            #plural_tables
            #direction_table
            #formatting_tables
        },
        plural_locales,
        plural_categories,
        formatting,
    })
}

fn generate_plural_tables(source: &str) -> Result<(TokenStream, usize, usize), String> {
    let value: Value = serde_json::from_str(source).map_err(|error| error.to_string())?;
    let cardinal = value
        .get("supplemental")
        .and_then(|value| value.get("plurals-type-cardinal"))
        .and_then(Value::as_object)
        .ok_or_else(|| "missing supplemental.plurals-type-cardinal".to_owned())?;

    let mut locales: Vec<_> = cardinal.iter().collect();
    locales.sort_by_key(|(left, _)| *left);

    let mut compiled_match_arms = Vec::new();
    let mut source_match_arms = Vec::new();
    let mut category_tables = Vec::new();
    let mut predicate_functions = Vec::new();
    let mut source_functions = Vec::new();
    let plural_locales = locales.len();
    let mut plural_categories = 0;

    for (locale, value) in locales {
        let object = value
            .as_object()
            .ok_or_else(|| format!("plural rules for locale `{locale}` are not an object"))?;
        let mut categories = Vec::new();
        for (key, value) in object {
            let Some(category) = key.strip_prefix("pluralRule-count-") else {
                continue;
            };
            let rule_source = value.as_str().ok_or_else(|| {
                format!("plural rule `{key}` for locale `{locale}` is not a string")
            })?;
            let rule = parse_plural_rule(rule_source)
                .map_err(|error| format!("{locale}/{category}: {error}"))?;
            categories.push((category.to_owned(), rule_source.to_owned(), rule));
        }
        if categories.is_empty() {
            return Err(format!("locale `{locale}` has no plural categories"));
        }
        plural_categories += categories.len();
        categories.sort_by(|left, right| {
            category_rank(&left.0)
                .cmp(&category_rank(&right.0))
                .then_with(|| left.0.cmp(&right.0))
        });

        let category_const = format_ident!("PLURAL_CATEGORIES_{}", const_name(locale));
        let source_function = format_ident!("plural_rule_source_{}", rust_identifier(locale));
        compiled_match_arms.push(quote! {
            #locale => Some(CompiledPluralRules {
                locale: #locale,
                categories: #category_const,
            }),
        });
        source_match_arms.push(quote! { #locale => Some(#source_function()), });

        let mut category_entries = Vec::new();
        let mut source_entries = Vec::new();
        for (category, rule_source, rule) in &categories {
            let function = format_ident!(
                "plural_{}_{}",
                rust_identifier(locale),
                rust_identifier(category)
            );
            category_entries.push(quote! {
                CompiledPluralCategory { category: #category, matches: #function }
            });
            let rule_literal = plural_rule_tokens(rule);
            source_entries.push(quote! {
                PluralCategoryRule {
                    category: #category.to_owned(),
                    source: #rule_source.to_owned(),
                    rule: #rule_literal,
                }
            });
            let parameter = if rule.conditions.is_empty() {
                format_ident!("_operands")
            } else {
                format_ident!("operands")
            };
            let body = plural_rule_match_tokens(rule, &parameter);
            predicate_functions.push(quote! {
                fn #function(#parameter: &PluralOperands) -> bool {
                    #body
                }
            });
        }

        category_tables.push(quote! {
            const #category_const: &[CompiledPluralCategory] = &[#(#category_entries),*];
        });
        source_functions.push(quote! {
            fn #source_function() -> PluralRules {
                PluralRules {
                    locale: #locale.to_owned(),
                    categories: vec![#(#source_entries),*],
                }
            }
        });
    }

    Ok((
        quote! {
            fn generated_plural_rules(locale: &str) -> Option<CompiledPluralRules> {
                match locale {
                    #(#compiled_match_arms)*
                    _ => None,
                }
            }

            fn generated_plural_rule_sources(locale: &str) -> Option<PluralRules> {
                match locale {
                    #(#source_match_arms)*
                    _ => None,
                }
            }

            fn integer_value(value: (f64, bool)) -> Option<u64> {
                if value.1 && value.0 >= 0.0 {
                    Some(value.0 as u64)
                } else {
                    None
                }
            }

            #(#category_tables)*
            #(#source_functions)*
            #(#predicate_functions)*
        },
        plural_locales,
        plural_categories,
    ))
}

fn plural_rule_tokens(rule: &PluralRule) -> TokenStream {
    let conditions = rule.conditions.iter().map(condition_tokens);
    quote! { PluralRule { conditions: vec![#(#conditions),*] } }
}

fn condition_tokens(condition: &Condition) -> TokenStream {
    let relations = condition.relations.iter().map(relation_tokens);
    quote! { Condition { relations: vec![#(#relations),*] } }
}

fn relation_tokens(relation: &Relation) -> TokenStream {
    let expression = operand_expression_tokens(&relation.expression);
    let operator = relation_operator_tokens(relation.operator);
    let ranges = range_list_tokens(&relation.ranges);
    quote! { Relation { expression: #expression, operator: #operator, ranges: #ranges } }
}

fn operand_expression_tokens(expression: &OperandExpression) -> TokenStream {
    let operand = operand_tokens(expression.operand);
    let modulo = option_u64_tokens(expression.modulo);
    quote! { OperandExpression { operand: #operand, modulo: #modulo } }
}

fn range_list_tokens(ranges: &RangeList) -> TokenStream {
    let ranges = ranges.ranges.iter().map(|range| {
        let start = range.start;
        let end = range.end;
        quote! { Range { start: #start, end: #end } }
    });
    quote! { RangeList { ranges: vec![#(#ranges),*] } }
}

fn option_u64_tokens(value: Option<u64>) -> TokenStream {
    value.map_or_else(|| quote! { None }, |value| quote! { Some(#value) })
}

fn operand_tokens(operand: Operand) -> TokenStream {
    match operand {
        Operand::N => quote! { Operand::N },
        Operand::I => quote! { Operand::I },
        Operand::V => quote! { Operand::V },
        Operand::W => quote! { Operand::W },
        Operand::F => quote! { Operand::F },
        Operand::T => quote! { Operand::T },
        Operand::C => quote! { Operand::C },
        Operand::E => quote! { Operand::E },
    }
}

fn relation_operator_tokens(operator: RelationOperator) -> TokenStream {
    match operator {
        RelationOperator::Equal => quote! { RelationOperator::Equal },
        RelationOperator::NotEqual => quote! { RelationOperator::NotEqual },
        RelationOperator::In => quote! { RelationOperator::In },
        RelationOperator::NotIn => quote! { RelationOperator::NotIn },
        RelationOperator::Within => quote! { RelationOperator::Within },
        RelationOperator::NotWithin => quote! { RelationOperator::NotWithin },
    }
}

fn plural_rule_match_tokens(rule: &PluralRule, operands: &proc_macro2::Ident) -> TokenStream {
    if rule.conditions.is_empty() {
        return quote! { true };
    }
    let conditions = rule
        .conditions
        .iter()
        .map(|condition| condition_match_tokens(condition, operands));
    join_boolean_tokens(conditions, false)
}

fn condition_match_tokens(condition: &Condition, operands: &proc_macro2::Ident) -> TokenStream {
    let relations = condition
        .relations
        .iter()
        .map(|relation| relation_match_tokens(relation, operands));
    join_boolean_tokens(relations, true)
}

fn relation_match_tokens(relation: &Relation, operands: &proc_macro2::Ident) -> TokenStream {
    let expression = operand_expression_value_tokens(&relation.expression, operands);
    match relation.operator {
        RelationOperator::Equal | RelationOperator::In => {
            let ranges = integer_ranges_match_tokens(&relation.ranges.ranges);
            quote! { integer_value(#expression).is_some_and(|value| #ranges) }
        }
        RelationOperator::NotEqual | RelationOperator::NotIn => {
            let ranges = integer_ranges_match_tokens(&relation.ranges.ranges);
            quote! { !integer_value(#expression).is_some_and(|value| #ranges) }
        }
        RelationOperator::Within => {
            let ranges = float_ranges_match_tokens(&relation.ranges.ranges);
            quote! { { let value = (#expression).0; value >= 0.0 && (#ranges) } }
        }
        RelationOperator::NotWithin => {
            let ranges = float_ranges_match_tokens(&relation.ranges.ranges);
            quote! { { let value = (#expression).0; !(value >= 0.0 && (#ranges)) } }
        }
    }
}

fn operand_expression_value_tokens(
    expression: &OperandExpression,
    operands: &proc_macro2::Ident,
) -> TokenStream {
    let operand = match expression.operand {
        Operand::N => quote! {
            (#operands.n.parse::<f64>().unwrap_or(#operands.i as f64), #operands.v == 0)
        },
        Operand::I => quote! { (#operands.i as f64, true) },
        Operand::V => quote! { (#operands.v as f64, true) },
        Operand::W => quote! { (#operands.w as f64, true) },
        Operand::F => quote! { (#operands.f as f64, true) },
        Operand::T => quote! { (#operands.t as f64, true) },
        Operand::C => quote! { (#operands.c as f64, true) },
        Operand::E => quote! { (#operands.e as f64, true) },
    };
    expression.modulo.map_or(operand.clone(), |modulo| {
        let modulo = proc_macro2::Literal::f64_unsuffixed(modulo as f64);
        quote! { { let value = #operand; (value.0 % #modulo, value.1) } }
    })
}

fn integer_ranges_match_tokens(ranges: &[Range]) -> TokenStream {
    let ranges = ranges.iter().map(|range| {
        let start = range.start;
        let end = range.end;
        if start == end {
            quote! { value == #start }
        } else {
            quote! { (#start..=#end).contains(&value) }
        }
    });
    join_boolean_tokens(ranges, false)
}

fn float_ranges_match_tokens(ranges: &[Range]) -> TokenStream {
    let ranges = ranges.iter().map(|range| {
        let start_value = range.start;
        let end_value = range.end;
        let start = proc_macro2::Literal::f64_unsuffixed(start_value as f64);
        let end = proc_macro2::Literal::f64_unsuffixed(end_value as f64);
        if start_value == end_value {
            quote! { value == #start }
        } else {
            quote! { (#start..=#end).contains(&value) }
        }
    });
    join_boolean_tokens(ranges, false)
}

fn join_boolean_tokens(
    tokens: impl IntoIterator<Item = TokenStream>,
    empty_value: bool,
) -> TokenStream {
    let mut tokens = tokens.into_iter();
    let Some(first) = tokens.next() else {
        return if empty_value {
            quote! { true }
        } else {
            quote! { false }
        };
    };
    tokens.fold(first, |left, right| {
        if empty_value {
            quote! { (#left) && (#right) }
        } else {
            quote! { (#left) || (#right) }
        }
    })
}

fn category_rank(category: &str) -> usize {
    match category {
        "zero" => 0,
        "one" => 1,
        "two" => 2,
        "few" => 3,
        "many" => 4,
        "other" => 5,
        _ => 6,
    }
}

fn const_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn rust_identifier(value: &str) -> String {
    let mut identifier = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            identifier.push(character.to_ascii_lowercase());
        } else {
            identifier.push('_');
        }
    }
    if identifier
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        identifier.insert(0, '_');
    }
    identifier
}
