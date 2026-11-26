//! Structured module dependencies share one syntax renderer and literal escaping.

use super::{
    EcmaImport, EcmaImportBindings, EcmaModuleKind, EcmaNamedImport, EcmaReExport,
    EcmaReExportBindings,
};
use minijinja::{context, Value};

fn named_bindings(values: &[EcmaNamedImport], types: &[EcmaNamedImport]) -> Vec<Value> {
    values
        .iter()
        .map(|binding| (binding, false))
        .chain(types.iter().map(|binding| (binding, true)))
        .map(|(binding, type_only)| {
            context! { imported => &binding.imported, local => &binding.local, type_only }
        })
        .collect()
}

pub(super) fn render_import(
    item: &EcmaImport,
    target: EcmaModuleKind,
    output: &mut String,
) -> bool {
    if !target.is_declaration() && matches!(item.bindings, EcmaImportBindings::TypeNamed(_)) {
        return false;
    }
    let bindings = match &item.bindings {
        EcmaImportBindings::SideEffect => context! { kind => "side_effect", type_only => false },
        EcmaImportBindings::Default(local) => {
            context! { kind => "default", local, type_only => false }
        }
        EcmaImportBindings::Namespace(local) => {
            context! { kind => "namespace", local, type_only => false }
        }
        EcmaImportBindings::Named(names) | EcmaImportBindings::TypeNamed(names) => {
            context! {
                kind => "named", names => named_bindings(names, &[]),
                type_only => matches!(item.bindings, EcmaImportBindings::TypeNamed(_)),
            }
        }
        EcmaImportBindings::NamedWithTypes { values, types } => {
            context! { kind => "named", names => named_bindings(values, types), type_only => false }
        }
    };
    output.push_str(&crate::templates::render(
        "module/edge",
        context! { statement => "import", specifier => &item.specifier, bindings },
    ));
    true
}

pub(super) fn render_re_export(
    item: &EcmaReExport,
    target: EcmaModuleKind,
    output: &mut String,
) -> bool {
    let type_only = matches!(
        item.bindings,
        EcmaReExportBindings::TypeAll | EcmaReExportBindings::TypeNamed(_)
    );
    if !target.is_declaration() && type_only {
        return false;
    }
    let bindings = match &item.bindings {
        EcmaReExportBindings::All | EcmaReExportBindings::TypeAll => {
            context! { kind => "all", type_only }
        }
        EcmaReExportBindings::Named(names) | EcmaReExportBindings::TypeNamed(names) => {
            context! { kind => "named", names => named_bindings(names, &[]), type_only }
        }
    };
    output.push_str(&crate::templates::render(
        "module/edge",
        context! { statement => "export", specifier => &item.specifier, bindings },
    ));
    true
}
