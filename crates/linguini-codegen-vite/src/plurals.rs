//! Serialize compiler-owned plural variants without interpreting CLDR in the Vite backend.

use std::collections::BTreeMap;

use linguini_codegen_ecmascript::{compile_plural_runtime_variants, EcmaGeneratedFile};

use crate::{ViteCodegenError, VitePlan};

pub(super) fn plural_artifacts(
    plan: &VitePlan<'_, '_>,
    files: &mut Vec<EcmaGeneratedFile>,
) -> Result<BTreeMap<String, serde_json::Value>, ViteCodegenError> {
    let mut runtimes = BTreeMap::new();
    for runtime in &plan.runtimes {
        let messages = plan
            .messages
            .iter()
            .filter(|message| message.locale == runtime.locale)
            .map(|message| message.message.clone())
            .collect::<Vec<_>>();
        let Some(compiled) = compile_plural_runtime_variants(plan.project, runtime, &messages)?
        else {
            continue;
        };
        let variants = compiled.variants.into_iter().map(|variant| {
            let entry = serde_json::json!({ "categories": variant.categories, "module": variant.file.path });
            files.push(variant.file);
            entry
        }).collect::<Vec<_>>();
        runtimes.insert(
            runtime.locale.clone(),
            serde_json::json!({
                "module": runtime.module_path,
                "binding": compiled.binding,
                "categories": compiled.categories,
                "messages": compiled.messages,
                "variants": variants,
            }),
        );
    }
    Ok(runtimes)
}
