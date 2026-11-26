//! Locale declaration overlays, independent of target syntax and artifact layout.

use linguini_ir::{
    IrEnum, IrForm, IrFunction, IrGroup, IrMessage, IrModuleBuilder, IrOrigin, IrVariable, LocaleIr,
};

/// Compose locale modules in increasing priority order (base first).
///
/// Later declarations replace earlier declarations of the same kind and name while
/// retaining their original vector position. Whole declarations are replaced; nested
/// form variants are not merged. All source origins remain, with replacements marked
/// as overrides. The caller validates references against the composed project schema.
pub fn compose_locale_modules<'a>(modules: impl IntoIterator<Item = &'a LocaleIr>) -> LocaleIr {
    let mut declarations = FallbackDeclarations::default();
    for module in modules {
        declarations.absorb(module);
    }
    declarations.finish()
}

/// Accumulates locale-fallback precedence: the first chain entry supplying a
/// name wins its vector position, and later duplicates replace that entry in
/// place instead of appending.
#[derive(Default)]
struct FallbackDeclarations {
    enums: Vec<IrEnum>,
    groups: Vec<IrGroup>,
    messages: Vec<IrMessage>,
    variables: Vec<IrVariable>,
    forms: Vec<IrForm>,
    functions: Vec<IrFunction>,
    origins: Vec<IrOrigin>,
}

impl FallbackDeclarations {
    fn absorb(&mut self, source: &LocaleIr) {
        macro_rules! upsert_kind {
            ($field:ident, $accessor:ident) => {
                for item in source.$accessor() {
                    match self
                        .$field
                        .iter_mut()
                        .find(|existing| existing.name == item.name)
                    {
                        Some(existing) => *existing = item.clone(),
                        None => self.$field.push(item.clone()),
                    }
                }
            };
        }

        upsert_kind!(enums, enums);
        upsert_kind!(groups, groups);
        upsert_kind!(messages, messages);
        upsert_kind!(variables, variables);
        upsert_kind!(forms, forms);
        upsert_kind!(functions, functions);

        for source_origin in source.origins() {
            let mut origin = source_origin.clone();
            if self
                .origins
                .iter()
                .any(|existing| existing.kind == origin.kind && existing.name == origin.name)
            {
                // Locale fallback precedence is an explicit semantic replacement. Retain both
                // provenance records while making that replacement visible to IR validation.
                origin.is_override = true;
            }
            self.origins.push(origin);
        }
    }

    fn finish(self) -> LocaleIr {
        let mut builder = IrModuleBuilder::new();
        for item in self.enums {
            builder = builder.push_enum(item);
        }
        for item in self.groups {
            builder = builder.push_group(item);
        }
        for item in self.messages {
            builder = builder.push_message(item);
        }
        for item in self.variables {
            builder = builder.push_variable(item);
        }
        for item in self.forms {
            builder = builder.push_form(item);
        }
        for item in self.functions {
            builder = builder.push_function(item);
        }
        for origin in self.origins {
            builder = builder.push_origin(origin);
        }
        LocaleIr::try_from_module(
            builder
                .build()
                .expect("upserted declarations have unique names per kind"),
        )
        .expect("locale composition cannot introduce type aliases")
    }
}
