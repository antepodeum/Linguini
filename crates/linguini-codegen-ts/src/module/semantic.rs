//! Per-symbol ESM artifacts used by physical bundler message leaves.

use std::collections::{BTreeMap, BTreeSet};

use linguini_ir::{
    is_plural_intrinsic, IrExpression, IrExpressionKind, IrFormEntry, IrFunctionBranch,
    IrFunctionBranchValue, IrFunctionParameter, IrInlineFunctionInput, IrModule, IrSymbolKind,
    IrText, IrTextPart, IrValue,
};
use linguini_syntax::{SourceId, Span};

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaNamedImport, EcmaSource, EcmaStatement,
};

use super::emit::{self, emit_forms, emit_local_functions, emit_variables};
use super::formatters::{formatter_requirements, plural_required};
use super::names::{form_binding_name, safe_identifier};
use super::{project_locale_options, TypeScriptCodegenError, ValidatedTypeScriptProject};

const MAX_PORTABLE_PATH_BYTES: usize = 240;
const MAX_ENCODED_NAME_BYTES: usize = 96;

/// Stable semantic symbol class. Ordering is part of artifact determinism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeScriptSemanticSymbolKind {
    LocaleEnum,
    LocaleVariable,
    LocaleForm,
    LocaleFunction,
}

impl TypeScriptSemanticSymbolKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocaleEnum => "enum",
            Self::LocaleVariable => "variable",
            Self::LocaleForm => "form",
            Self::LocaleFunction => "function",
        }
    }

    fn ir_kind(self) -> IrSymbolKind {
        match self {
            Self::LocaleEnum => IrSymbolKind::Enum,
            Self::LocaleVariable => IrSymbolKind::Variable,
            Self::LocaleForm => IrSymbolKind::Form,
            Self::LocaleFunction => IrSymbolKind::Function,
        }
    }
}

/// Exact direct semantic ESM edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeScriptSemanticImport {
    pub kind: TypeScriptSemanticSymbolKind,
    pub name: String,
    pub binding: String,
    pub module_path: String,
    pub import_path: String,
    pub source_ids: Vec<SourceId>,
    pub type_only: bool,
}

/// Stable descriptor for one generated semantic leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScriptSemanticArtifact {
    pub locale: String,
    pub kind: TypeScriptSemanticSymbolKind,
    pub name: String,
    pub binding: String,
    pub module_path: String,
    pub source_map_path: String,
    pub output_file_name: String,
    pub shared_import_path: String,
    pub runtime_module_path: String,
    pub runtime_import_path: String,
    pub source_ids: Vec<SourceId>,
    pub imports: Vec<TypeScriptSemanticImport>,
}

/// Rendered one-binding semantic module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTypeScriptSemanticModule {
    pub artifact: TypeScriptSemanticArtifact,
    pub code: String,
    pub source_map: String,
}

pub(super) fn semantic_artifacts(
    project: &ValidatedTypeScriptProject<'_>,
) -> Result<Vec<TypeScriptSemanticArtifact>, TypeScriptCodegenError> {
    let selected_messages =
        if project.options.tree_shaking && !project.options.included_messages.is_empty() {
            super::visible_schema(
                project.schema,
                &super::TypeScriptOptions {
                    included_messages: project.options.included_messages.clone(),
                    ..super::TypeScriptOptions::default()
                },
            )
        } else {
            project.schema.clone()
        };
    let message_names = selected_messages
        .messages
        .iter()
        .map(|message| message.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut required = BTreeSet::new();
    for locale in &project.locales {
        for message in &message_names {
            let closure = project.message_dependency_closure(&locale.locale, message)?;
            for item in &closure.locale_module().enums {
                required.insert((
                    locale.locale.clone(),
                    TypeScriptSemanticSymbolKind::LocaleEnum,
                    item.name.clone(),
                ));
            }
            for item in &closure.locale_module().variables {
                required.insert((
                    locale.locale.clone(),
                    TypeScriptSemanticSymbolKind::LocaleVariable,
                    item.name.clone(),
                ));
            }
            for item in &closure.locale_module().forms {
                required.insert((
                    locale.locale.clone(),
                    TypeScriptSemanticSymbolKind::LocaleForm,
                    item.name.clone(),
                ));
            }
            for item in &closure.locale_module().functions {
                required.insert((
                    locale.locale.clone(),
                    TypeScriptSemanticSymbolKind::LocaleFunction,
                    item.name.clone(),
                ));
            }
        }
    }

    let mut base = BTreeMap::new();
    for (locale, kind, name) in &required {
        let module_path = semantic_module_path(locale, *kind, name)?;
        let binding = semantic_binding(*kind, name);
        base.insert(
            (locale.clone(), *kind, name.clone()),
            (module_path, binding),
        );
    }
    validate_paths(&base)?;

    let mut artifacts = Vec::with_capacity(base.len());
    for ((locale, kind, name), (module_path, binding)) in &base {
        let locale_module = &project
            .locales
            .iter()
            .find(|entry| entry.locale == *locale)
            .expect("semantic locale comes from validated project")
            .module;
        let dependencies = direct_dependencies(project.schema, locale_module, *kind, name);
        let mut imports = Vec::new();
        for (dependency_kind, dependency_name) in dependencies {
            let key = (locale.clone(), dependency_kind, dependency_name.clone());
            let Some((dependency_path, dependency_binding)) = base.get(&key) else {
                continue;
            };
            imports.push(TypeScriptSemanticImport {
                kind: dependency_kind,
                name: dependency_name.clone(),
                binding: dependency_binding.clone(),
                module_path: dependency_path.clone(),
                import_path: relative_import(module_path, dependency_path),
                source_ids: symbol_source_ids(locale_module, dependency_kind, &dependency_name),
                type_only: dependency_kind == TypeScriptSemanticSymbolKind::LocaleEnum,
            });
        }
        imports.sort();
        let parent_depth = module_path.matches('/').count();
        let runtime_module_path = format!("locales/{locale}/_runtime.ts");
        artifacts.push(TypeScriptSemanticArtifact {
            locale: locale.clone(),
            kind: *kind,
            name: name.clone(),
            binding: binding.clone(),
            source_map_path: format!("{module_path}.map"),
            output_file_name: module_path
                .rsplit('/')
                .next()
                .expect("file name")
                .to_owned(),
            shared_import_path: format!("{}shared", "../".repeat(parent_depth)),
            runtime_import_path: relative_import(module_path, &runtime_module_path),
            runtime_module_path,
            source_ids: symbol_source_ids(locale_module, *kind, name),
            imports,
            module_path: module_path.clone(),
        });
    }
    validate_cycles(&artifacts)?;
    Ok(artifacts)
}

pub fn compile_typescript_bundler_semantic_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptSemanticArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptSemanticModule, TypeScriptCodegenError> {
    let locale = &project
        .locales
        .iter()
        .find(|entry| entry.locale == artifact.locale)
        .ok_or_else(|| TypeScriptCodegenError::UnknownLocale {
            locale: artifact.locale.clone(),
        })?
        .module;
    let options = project_locale_options(&artifact.locale, &project.options)?;
    let one = one_symbol_module(locale, artifact.kind, &artifact.name);
    if !module_has_symbol(&one, artifact.kind) {
        return Err(TypeScriptCodegenError::UnknownSemanticArtifact {
            locale: artifact.locale.clone(),
            kind: artifact.kind.as_str().to_owned(),
            name: artifact.name.clone(),
        });
    }
    let mut imports = artifact
        .imports
        .iter()
        .map(|dependency| EcmaImport {
            specifier: dependency.import_path.clone(),
            bindings: if dependency.type_only {
                EcmaImportBindings::TypeNamed(vec![EcmaNamedImport::new(
                    &dependency.binding,
                    &dependency.binding,
                )])
            } else {
                EcmaImportBindings::Named(vec![EcmaNamedImport::new(
                    &dependency.binding,
                    &dependency.binding,
                )])
            },
        })
        .collect::<Vec<_>>();

    let type_names = schema_types_for_symbol(project.schema, &one);
    if !type_names.is_empty() {
        imports.push(EcmaImport {
            specifier: artifact.shared_import_path.clone(),
            bindings: EcmaImportBindings::TypeNamed(
                type_names
                    .iter()
                    .map(|name| EcmaNamedImport::new(name, name))
                    .collect(),
            ),
        });
    }

    let mut output = String::new();
    let (declaration, exported_declaration) = match artifact.kind {
        TypeScriptSemanticSymbolKind::LocaleEnum => {
            emit::emit_locale_enum_types(project.schema, &one, &mut output);
            (
                format!("type {} =", artifact.binding),
                format!("export type {} =", artifact.binding),
            )
        }
        TypeScriptSemanticSymbolKind::LocaleVariable => {
            emit_variables(&one, &options, &mut output);
            (
                format!("const {} =", artifact.binding),
                format!("export const {} =", artifact.binding),
            )
        }
        TypeScriptSemanticSymbolKind::LocaleForm => {
            emit_forms(&one, &options, &mut output);
            (
                format!("const {} =", artifact.binding),
                format!("export const {} =", artifact.binding),
            )
        }
        TypeScriptSemanticSymbolKind::LocaleFunction => {
            emit_local_functions(&one, &options, &mut output);
            (
                format!("function {}(", artifact.binding),
                format!("export function {}(", artifact.binding),
            )
        }
    };
    output = output.replacen(&declaration, &exported_declaration, 1);

    let uses_select_branch = output.contains("selectBranch(");
    if uses_select_branch {
        imports.push(EcmaImport::named(
            &artifact.shared_import_path,
            vec![EcmaNamedImport::new("selectBranch", "selectBranch")],
        ));
    }
    let mut helpers = formatter_requirements(&IrModule::default(), &one).helper_names();
    if plural_required(&IrModule::default(), &one) {
        helpers.push(&options.plural_function);
    }
    if !helpers.is_empty() {
        imports.push(EcmaImport::named(
            &artifact.runtime_import_path,
            helpers
                .into_iter()
                .map(|name| EcmaNamedImport::new(name, name))
                .collect(),
        ));
    }
    imports.sort_by(|left, right| left.specifier.cmp(&right.specifier));

    let span = symbol_span(locale, artifact.kind, &artifact.name);
    let statement = match span {
        Some(span) => EcmaStatement::mapped(output, span),
        None => EcmaStatement::generated(output),
    };
    let module = EcmaModule {
        imports,
        statements: vec![statement],
    };
    let source_records = ordered_sources(&artifact.source_ids, sources)?;
    let rendered = module.render(&artifact.output_file_name, &source_records);
    Ok(CompiledTypeScriptSemanticModule {
        artifact: artifact.clone(),
        code: rendered.code,
        source_map: rendered.source_map,
    })
}

pub(super) fn message_imports_from_shared(
    project: &ValidatedTypeScriptProject<'_>,
    locale: &str,
    message: &str,
    shared_import_path: &str,
) -> Result<Vec<TypeScriptSemanticImport>, TypeScriptCodegenError> {
    let artifacts = semantic_artifacts(project)?;
    let root = shared_import_path
        .strip_suffix("/shared")
        .or_else(|| (shared_import_path == "shared").then_some("."))
        .unwrap_or(shared_import_path);
    message_imports_with_artifacts(project, locale, message, &artifacts, |artifact| {
        format!(
            "{}/{}",
            root.trim_end_matches('/'),
            artifact.module_path.trim_end_matches(".ts")
        )
    })
}

pub(super) fn message_imports_with_artifacts(
    project: &ValidatedTypeScriptProject<'_>,
    locale: &str,
    message: &str,
    artifacts: &[TypeScriptSemanticArtifact],
    import_path: impl Fn(&TypeScriptSemanticArtifact) -> String,
) -> Result<Vec<TypeScriptSemanticImport>, TypeScriptCodegenError> {
    let locale_module = &project
        .locales
        .iter()
        .find(|entry| entry.locale == locale)
        .ok_or_else(|| TypeScriptCodegenError::UnknownLocale {
            locale: locale.to_owned(),
        })?
        .module;
    let signature = project
        .schema
        .messages
        .iter()
        .find(|item| item.name == message)
        .ok_or_else(|| TypeScriptCodegenError::UnknownMessage {
            message: message.to_owned(),
        })?;
    let implementation = locale_module
        .messages
        .iter()
        .find(|item| item.name == message)
        .ok_or_else(|| TypeScriptCodegenError::MissingMessageImplementation {
            locale: locale.to_owned(),
            message: message.to_owned(),
        })?;
    let mut collector = ReferenceCollector::new(project.schema, locale_module);
    let context = signature
        .parameters
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    for parameter in &signature.parameters {
        collector.record_type(&parameter.ty);
    }
    if let Some(body) = &implementation.body {
        collector.text(body, &context);
    }
    Ok(collector
        .dependencies
        .into_iter()
        .filter_map(|(kind, name)| {
            artifacts
                .iter()
                .find(|item| item.locale == locale && item.kind == kind && item.name == name)
                .map(|item| TypeScriptSemanticImport {
                    kind,
                    name,
                    binding: item.binding.clone(),
                    module_path: item.module_path.clone(),
                    import_path: import_path(item),
                    source_ids: item.source_ids.clone(),
                    type_only: kind == TypeScriptSemanticSymbolKind::LocaleEnum,
                })
        })
        .collect())
}

fn semantic_binding(kind: TypeScriptSemanticSymbolKind, name: &str) -> String {
    match kind {
        TypeScriptSemanticSymbolKind::LocaleForm => form_binding_name(name),
        _ => safe_identifier(name),
    }
}

fn semantic_module_path(
    locale: &str,
    kind: TypeScriptSemanticSymbolKind,
    name: &str,
) -> Result<String, TypeScriptCodegenError> {
    let encoded = encoded_path(name.as_bytes());
    let path = format!("bundler/semantic/{locale}/{}/{encoded}.ts", kind.as_str());
    if path.len() > MAX_PORTABLE_PATH_BYTES {
        return Err(TypeScriptCodegenError::InvalidSemanticArtifactPath {
            locale: locale.to_owned(),
            kind: kind.as_str().to_owned(),
            name: name.to_owned(),
            reason: "encoded path exceeds 240 bytes",
        });
    }
    Ok(path)
}

fn encoded_path(value: &[u8]) -> String {
    let mut encoded = String::from("x");
    for byte in value {
        use std::fmt::Write;
        write!(encoded, "{byte:02x}").expect("String write");
    }
    if encoded.len() <= MAX_ENCODED_NAME_BYTES {
        return encoded;
    }
    let hash = fnv1a(value);
    format!("{}-h{hash:016x}", &encoded[..MAX_ENCODED_NAME_BYTES - 18])
}

fn fnv1a(value: &[u8]) -> u64 {
    value.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

pub(super) fn relative_import(from: &str, to: &str) -> String {
    let from_parts = from.split('/').collect::<Vec<_>>();
    let mut to_parts = to.trim_end_matches(".ts").split('/').collect::<Vec<_>>();
    let from_dirs = &from_parts[..from_parts.len() - 1];
    let common = from_dirs
        .iter()
        .zip(&to_parts)
        .take_while(|(a, b)| a == b)
        .count();
    let mut result = "../".repeat(from_dirs.len() - common);
    if result.is_empty() {
        result.push_str("./");
    }
    result.push_str(&to_parts.drain(common..).collect::<Vec<_>>().join("/"));
    result
}

fn validate_paths(
    base: &BTreeMap<(String, TypeScriptSemanticSymbolKind, String), (String, String)>,
) -> Result<(), TypeScriptCodegenError> {
    let mut paths = BTreeMap::new();
    for ((locale, kind, name), (path, _)) in base {
        if let Some(conflict) = paths.insert(path.to_ascii_lowercase(), (locale, kind, name)) {
            return Err(TypeScriptCodegenError::OutputPathCollision {
                path: path.clone(),
                conflicts_with: format!(
                    "semantic {} `{}` for locale `{}`",
                    conflict.1.as_str(),
                    conflict.2,
                    conflict.0
                ),
            });
        }
    }
    Ok(())
}

fn validate_cycles(artifacts: &[TypeScriptSemanticArtifact]) -> Result<(), TypeScriptCodegenError> {
    let by_path = artifacts
        .iter()
        .map(|item| (item.module_path.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    fn visit<'a>(
        path: &'a str,
        by_path: &BTreeMap<&'a str, &'a TypeScriptSemanticArtifact>,
        active: &mut Vec<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Option<Vec<String>> {
        if let Some(index) = active.iter().position(|item| *item == path) {
            return Some(
                active[index..]
                    .iter()
                    .map(|item| (*item).to_owned())
                    .chain(std::iter::once(path.to_owned()))
                    .collect(),
            );
        }
        if done.contains(path) {
            return None;
        }
        active.push(path);
        if let Some(item) = by_path.get(path) {
            for dependency in &item.imports {
                if dependency.type_only {
                    continue;
                }
                if let Some(cycle) = visit(&dependency.module_path, by_path, active, done) {
                    return Some(cycle);
                }
            }
        }
        active.pop();
        done.insert(path);
        None
    }
    let mut done = BTreeSet::new();
    for artifact in artifacts {
        if let Some(cycle) = visit(&artifact.module_path, &by_path, &mut Vec::new(), &mut done) {
            return Err(TypeScriptCodegenError::SemanticDependencyCycle {
                locale: artifact.locale.clone(),
                cycle,
            });
        }
    }
    Ok(())
}

fn one_symbol_module(
    locale: &IrModule,
    kind: TypeScriptSemanticSymbolKind,
    name: &str,
) -> IrModule {
    let mut one = IrModule::default();
    match kind {
        TypeScriptSemanticSymbolKind::LocaleEnum => one.enums.extend(
            locale
                .enums
                .iter()
                .filter(|item| item.name == name)
                .cloned(),
        ),
        TypeScriptSemanticSymbolKind::LocaleVariable => one.variables.extend(
            locale
                .variables
                .iter()
                .filter(|item| item.name == name)
                .cloned(),
        ),
        TypeScriptSemanticSymbolKind::LocaleForm => one.forms.extend(
            locale
                .forms
                .iter()
                .filter(|item| item.name == name)
                .cloned(),
        ),
        TypeScriptSemanticSymbolKind::LocaleFunction => one.functions.extend(
            locale
                .functions
                .iter()
                .filter(|item| item.name == name)
                .cloned(),
        ),
    }
    one.origins.extend(
        locale
            .origins
            .iter()
            .filter(|origin| origin.kind == kind.ir_kind() && origin.name == name)
            .cloned(),
    );
    one
}

fn module_has_symbol(module: &IrModule, kind: TypeScriptSemanticSymbolKind) -> bool {
    match kind {
        TypeScriptSemanticSymbolKind::LocaleEnum => !module.enums.is_empty(),
        TypeScriptSemanticSymbolKind::LocaleVariable => !module.variables.is_empty(),
        TypeScriptSemanticSymbolKind::LocaleForm => !module.forms.is_empty(),
        TypeScriptSemanticSymbolKind::LocaleFunction => !module.functions.is_empty(),
    }
}

fn symbol_span(locale: &IrModule, kind: TypeScriptSemanticSymbolKind, name: &str) -> Option<Span> {
    locale
        .origins
        .iter()
        .rev()
        .find(|origin| origin.kind == kind.ir_kind() && origin.name == name)
        .map(|origin| origin.span)
}

fn symbol_source_ids(
    locale: &IrModule,
    kind: TypeScriptSemanticSymbolKind,
    name: &str,
) -> Vec<SourceId> {
    let mut sources = BTreeSet::new();
    if let Some(span) = symbol_span(locale, kind, name) {
        sources.insert(span.source);
    }
    match kind {
        TypeScriptSemanticSymbolKind::LocaleEnum => {}
        TypeScriptSemanticSymbolKind::LocaleVariable => {
            if let Some(item) = locale.variables.iter().find(|item| item.name == name) {
                collect_text_source_ids(&item.value, &mut sources);
            }
        }
        TypeScriptSemanticSymbolKind::LocaleForm => {
            if let Some(item) = locale.forms.iter().find(|item| item.name == name) {
                for variant in &item.variants {
                    collect_form_source_ids(&variant.entries, &mut sources);
                }
            }
        }
        TypeScriptSemanticSymbolKind::LocaleFunction => {
            if let Some(item) = locale.functions.iter().find(|item| item.name == name) {
                for branch in &item.branches {
                    collect_function_source_ids(branch, &mut sources);
                }
            }
        }
    }
    sources.into_iter().collect()
}

fn collect_text_source_ids(text: &IrText, output: &mut BTreeSet<SourceId>) {
    output.insert(text.span.source);
    for part in &text.parts {
        if let IrTextPart::Placeholder(expression) = part {
            collect_expression_source_ids(expression, output);
        }
    }
}

fn collect_expression_source_ids(expression: &IrExpression, output: &mut BTreeSet<SourceId>) {
    output.insert(expression.span.source);
    for argument in &expression.arguments {
        collect_expression_source_ids(argument, output);
    }
    if let IrExpressionKind::InlineFunction { inputs, branches } = &expression.kind {
        for input in inputs {
            let (value, span) = match input {
                IrInlineFunctionInput::Binding { value, span, .. }
                | IrInlineFunctionInput::Selector { value, span } => (value, span),
            };
            output.insert(span.source);
            collect_expression_source_ids(value, output);
        }
        for branch in branches {
            collect_function_source_ids(branch, output);
        }
    }
}

fn collect_function_source_ids(branch: &IrFunctionBranch, output: &mut BTreeSet<SourceId>) {
    output.insert(branch.span.source);
    match &branch.value {
        IrFunctionBranchValue::Text(text) => collect_text_source_ids(text, output),
        IrFunctionBranchValue::Dispatch(branches) => {
            for branch in branches {
                collect_function_source_ids(branch, output);
            }
        }
    }
}

fn collect_form_source_ids(entries: &[IrFormEntry], output: &mut BTreeSet<SourceId>) {
    for entry in entries {
        match entry {
            IrFormEntry::Attribute { value, .. } => collect_value_source_ids(value, output),
            IrFormEntry::Branch(branch) => {
                output.insert(branch.span.source);
                collect_text_source_ids(&branch.value, output);
            }
        }
    }
}

fn collect_value_source_ids(value: &IrValue, output: &mut BTreeSet<SourceId>) {
    match value {
        IrValue::Text(text) => collect_text_source_ids(text, output),
        IrValue::Map(branches) => {
            for branch in branches {
                output.insert(branch.span.source);
                collect_text_source_ids(&branch.value, output);
            }
        }
        IrValue::Object(entries) => collect_form_source_ids(entries, output),
    }
}

fn ordered_sources(
    ids: &[SourceId],
    sources: &[EcmaSource],
) -> Result<Vec<EcmaSource>, TypeScriptCodegenError> {
    let mut records = BTreeMap::new();
    for source in sources {
        if records.insert(source.id, source).is_some() {
            return Err(TypeScriptCodegenError::DuplicateMessageSource {
                source_id: source.id,
            });
        }
    }
    ids.iter()
        .map(|id| {
            records
                .get(id)
                .map(|item| (*item).clone())
                .ok_or(TypeScriptCodegenError::MissingMessageSource { source_id: *id })
        })
        .collect()
}

fn schema_types_for_symbol(schema: &IrModule, locale: &IrModule) -> Vec<String> {
    let mut names = BTreeSet::new();
    for function in &locale.functions {
        for parameter in &function.parameters {
            collect_schema_type(schema, &parameter.ty, &mut names);
        }
    }
    for form in &locale.forms {
        for variant in &form.variants {
            collect_form_types(schema, &variant.entries, &mut names);
        }
    }
    names
        .into_iter()
        .map(|name| safe_identifier(&name))
        .collect()
}

fn collect_schema_type(schema: &IrModule, ty: &str, names: &mut BTreeSet<String>) {
    if let Some(alias) = schema.type_aliases.iter().find(|item| item.name == ty) {
        if names.insert(alias.name.clone()) {
            collect_schema_type(schema, &alias.target, names);
        }
    } else if schema.enums.iter().any(|item| item.name == ty) {
        names.insert(ty.to_owned());
    }
}
fn collect_form_types(schema: &IrModule, entries: &[IrFormEntry], names: &mut BTreeSet<String>) {
    for entry in entries {
        if let IrFormEntry::Attribute {
            parameters, value, ..
        } = entry
        {
            for parameter in parameters {
                collect_schema_type(schema, &parameter.ty, names);
            }
            if let IrValue::Object(children) = value {
                collect_form_types(schema, children, names);
            }
        }
    }
}

fn direct_dependencies(
    schema: &IrModule,
    locale: &IrModule,
    kind: TypeScriptSemanticSymbolKind,
    name: &str,
) -> BTreeSet<(TypeScriptSemanticSymbolKind, String)> {
    let mut collector = ReferenceCollector::new(schema, locale);
    match kind {
        TypeScriptSemanticSymbolKind::LocaleEnum => {}
        TypeScriptSemanticSymbolKind::LocaleVariable => {
            if let Some(item) = locale.variables.iter().find(|item| item.name == name) {
                collector.text(&item.value, &BTreeMap::new());
            }
        }
        TypeScriptSemanticSymbolKind::LocaleForm => {
            if let Some(item) = locale.forms.iter().find(|item| item.name == name) {
                for variant in &item.variants {
                    collector.form_entries(&variant.entries, &BTreeMap::new());
                }
            }
        }
        TypeScriptSemanticSymbolKind::LocaleFunction => {
            if let Some(item) = locale.functions.iter().find(|item| item.name == name) {
                for parameter in &item.parameters {
                    collector.record_type(&parameter.ty);
                }
                let context = parameter_context(&item.parameters);
                for branch in &item.branches {
                    collector.function_branch(branch, &context);
                }
            }
        }
    }
    collector.dependencies.remove(&(kind, name.to_owned()));
    collector.dependencies
}

fn parameter_context(parameters: &[IrFunctionParameter]) -> BTreeMap<String, String> {
    parameters
        .iter()
        .filter_map(|parameter| {
            parameter
                .name
                .as_ref()
                .map(|name| (name.clone(), parameter.ty.clone()))
        })
        .collect()
}

struct ReferenceCollector<'a> {
    schema: &'a IrModule,
    locale: &'a IrModule,
    dependencies: BTreeSet<(TypeScriptSemanticSymbolKind, String)>,
}
impl<'a> ReferenceCollector<'a> {
    fn new(schema: &'a IrModule, locale: &'a IrModule) -> Self {
        Self {
            schema,
            locale,
            dependencies: BTreeSet::new(),
        }
    }
    fn record_type(&mut self, ty: &str) {
        let resolved = self.resolve_type(ty);
        if self.locale.enums.iter().any(|item| item.name == resolved) {
            self.dependencies
                .insert((TypeScriptSemanticSymbolKind::LocaleEnum, resolved));
        }
    }
    fn text(&mut self, text: &IrText, context: &BTreeMap<String, String>) {
        for part in &text.parts {
            if let IrTextPart::Placeholder(expr) = part {
                self.expression(expr, context);
            }
        }
    }
    fn expression(&mut self, expr: &IrExpression, context: &BTreeMap<String, String>) {
        for arg in &expr.arguments {
            self.expression(arg, context);
        }
        if let IrExpressionKind::InlineFunction { inputs, branches } = &expr.kind {
            let mut inner = context.clone();
            for input in inputs {
                match input {
                    IrInlineFunctionInput::Binding { name, value, .. } => {
                        self.expression(value, &inner);
                        inner.insert(name.clone(), inferred_expression_type(value, &inner));
                    }
                    IrInlineFunctionInput::Selector { value, .. } => self.expression(value, &inner),
                }
            }
            for branch in branches {
                self.function_branch(branch, &inner);
            }
            return;
        }
        if expr.path.is_empty() {
            return;
        }
        let full = expr.path.join(".");
        if self.locale.functions.iter().any(|item| item.name == full) {
            self.dependencies
                .insert((TypeScriptSemanticSymbolKind::LocaleFunction, full));
            return;
        }
        if self.locale.forms.iter().any(|item| item.name == full) {
            self.dependencies
                .insert((TypeScriptSemanticSymbolKind::LocaleForm, full));
            return;
        }
        if let Some(ty) = context.get(&expr.path[0]) {
            if expr.path.len() > 1 || expr.kind == IrExpressionKind::Call {
                let resolved = self.resolve_type(ty);
                if self.locale.forms.iter().any(|item| item.name == resolved) {
                    self.dependencies
                        .insert((TypeScriptSemanticSymbolKind::LocaleForm, resolved));
                }
            }
        }
        if expr.kind == IrExpressionKind::Reference
            && expr.path.len() == 1
            && self
                .locale
                .variables
                .iter()
                .any(|item| item.name == expr.path[0])
        {
            self.dependencies.insert((
                TypeScriptSemanticSymbolKind::LocaleVariable,
                expr.path[0].clone(),
            ));
        }
    }
    fn resolve_type(&self, ty: &str) -> String {
        let mut current = ty.to_owned();
        let mut seen = BTreeSet::new();
        while seen.insert(current.clone()) {
            if let Some(alias) = self
                .schema
                .type_aliases
                .iter()
                .find(|item| item.name == current)
            {
                current = alias.target.clone();
            } else {
                break;
            }
        }
        current
    }
    fn function_branch(&mut self, branch: &IrFunctionBranch, context: &BTreeMap<String, String>) {
        match &branch.value {
            IrFunctionBranchValue::Text(text) => self.text(text, context),
            IrFunctionBranchValue::Dispatch(items) => {
                for item in items {
                    self.function_branch(item, context);
                }
            }
        }
    }
    fn form_entries(&mut self, entries: &[IrFormEntry], inherited: &BTreeMap<String, String>) {
        for entry in entries {
            match entry {
                IrFormEntry::Attribute {
                    parameters, value, ..
                } => {
                    for parameter in parameters {
                        self.record_type(&parameter.ty);
                    }
                    let mut context = inherited.clone();
                    context.extend(parameter_context(parameters));
                    self.value(value, &context);
                }
                IrFormEntry::Branch(branch) => self.text(&branch.value, inherited),
            }
        }
    }
    fn value(&mut self, value: &IrValue, context: &BTreeMap<String, String>) {
        match value {
            IrValue::Text(text) => self.text(text, context),
            IrValue::Map(branches) => {
                for branch in branches {
                    self.text(&branch.value, context)
                }
            }
            IrValue::Object(entries) => self.form_entries(entries, context),
        }
    }
}

fn inferred_expression_type(
    expression: &IrExpression,
    context: &BTreeMap<String, String>,
) -> String {
    match &expression.kind {
        IrExpressionKind::Reference if expression.path.len() == 1 => context
            .get(&expression.path[0])
            .cloned()
            .unwrap_or_else(|| "String".to_owned()),
        IrExpressionKind::Call
            if expression.path.len() == 1 && is_plural_intrinsic(&expression.path[0]) =>
        {
            "Plural".to_owned()
        }
        _ => "String".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compile_typescript_bundler_message_artifact_module,
        compile_typescript_bundler_message_module, EcmaSource, TypeScriptLocaleModule,
        TypeScriptProjectOptions,
    };
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in};

    fn project(schema: &str, locale: &str) -> ValidatedTypeScriptProject<'static> {
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in(schema, SourceId(11)).expect("schema"),
        )));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: lower_locale(&parse_locale_in(locale, SourceId(12)).expect("locale")),
        }]));
        ValidatedTypeScriptProject::try_new(
            schema,
            locales,
            &TypeScriptProjectOptions {
                base_locale: Some("en".to_owned()),
                ..TypeScriptProjectOptions::default()
            },
        )
        .expect("project")
    }

    fn sources(schema: &str, locale: &str) -> Vec<EcmaSource> {
        vec![
            EcmaSource::new(SourceId(11), "schema.lgs", schema),
            EcmaSource::new(SourceId(12), "en.lgl", locale),
        ]
    }

    #[test]
    fn shared_symbol_has_one_deterministic_artifact_and_physical_leaves_import_it() {
        let schema = "first\nsecond\n";
        let locale = "let shared = Common\nfirst = {shared}\nsecond = {shared}\n";
        let project = project(schema, locale);
        let artifacts = project.semantic_artifacts().expect("artifacts");
        assert_eq!(artifacts.len(), 1);
        let artifact = &artifacts[0];
        assert_eq!(artifact.kind, TypeScriptSemanticSymbolKind::LocaleVariable);
        assert_eq!(artifact.name, "shared");
        assert!(artifact
            .module_path
            .starts_with("bundler/semantic/en/variable/x"));
        assert_eq!(artifacts, project.semantic_artifacts().expect("repeat"));

        let message_artifacts = project.message_artifacts().expect("messages");
        assert!(message_artifacts
            .iter()
            .all(|message| message.semantic_imports
                == vec![TypeScriptSemanticImport {
                    kind: artifact.kind,
                    name: artifact.name.clone(),
                    binding: artifact.binding.clone(),
                    module_path: artifact.module_path.clone(),
                    import_path: relative_import(&message.module_path, &artifact.module_path),
                    source_ids: artifact.source_ids.clone(),
                    type_only: false,
                }]));

        let first = message_artifacts
            .iter()
            .find(|item| item.message == "first")
            .unwrap();
        let compiled = compile_typescript_bundler_message_artifact_module(
            &project,
            first,
            &sources(schema, locale),
        )
        .expect("message");
        assert!(compiled.code.contains(&format!(
            "import {{ shared }} from \"{}\";",
            first.semantic_imports[0].import_path
        )));
        assert!(!compiled.code.contains("const shared ="));

        let semantic = compile_typescript_bundler_semantic_module(
            &project,
            artifact,
            &sources(schema, locale),
        )
        .expect("semantic");
        assert!(semantic.code.contains("export const shared = \"Common\";"));
        assert_eq!(semantic.code.matches("export ").count(), 1);
        assert!(semantic.source_map.contains("en.lgl"));
        assert!(semantic.source_map.contains("\"mappings\":\"AAAA\""));

        let relocated = compile_typescript_bundler_message_module(
            &project,
            "en",
            "first",
            "leaf.ts",
            "../../shared",
            "../../locales/en/_runtime",
            &sources(schema, locale),
        )
        .expect("relocated legacy caller");
        assert!(relocated
            .code
            .contains("from \"../../bundler/semantic/en/variable/"));
    }

    #[test]
    fn semantic_modules_use_direct_edges_and_exact_runtime_helpers() {
        let schema = "root(value: Number)\n";
        let locale = "let prefix = Total\nfn Render(value: Number) { _ => {prefix}: {value @number} }\nroot = {Render(value)}\n";
        let project = project(schema, locale);
        let artifacts = project.semantic_artifacts().expect("artifacts");
        let function = artifacts
            .iter()
            .find(|item| item.kind == TypeScriptSemanticSymbolKind::LocaleFunction)
            .unwrap();
        assert_eq!(function.imports.len(), 1);
        assert_eq!(function.imports[0].name, "prefix");
        let compiled = compile_typescript_bundler_semantic_module(
            &project,
            function,
            &sources(schema, locale),
        )
        .expect("function");
        assert!(compiled.code.contains("import { prefix }"));
        assert!(compiled.code.contains("import { formatNumber }"));
        assert!(!compiled.code.contains("formatCurrency"));
        assert!(!compiled.code.contains("const prefix ="));
        assert_eq!(compiled.code.matches("export ").count(), 1);

        let message = project.message_artifacts().expect("messages").remove(0);
        let physical = compile_typescript_bundler_message_module(
            &project,
            "en",
            "root",
            &message.output_file_name,
            &message.shared_import_path,
            &message.runtime_import_path,
            &sources(schema, locale),
        )
        .expect("physical message");
        assert!(physical.code.contains("import { Render }"));
        assert!(!physical.code.contains("function Render("));
        assert!(!physical.code.contains("const prefix ="));
    }

    #[test]
    fn physical_form_is_imported_and_locale_enum_dependency_is_type_only() {
        let schema = "enum Fruit { apple }\nroot(fruit: Fruit)\n";
        let locale = "enum Gender { male, other }\nimpl Fruit { apple { Gender = male } }\nfn choose(Gender) {\n  male => Male\n  other => Other\n}\nroot = {choose(fruit.Gender)}\n";
        let project = project(schema, locale);
        let artifacts = project.semantic_artifacts().expect("artifacts");
        let form = artifacts
            .iter()
            .find(|item| item.kind == TypeScriptSemanticSymbolKind::LocaleForm)
            .expect("form");
        let locale_enum = artifacts
            .iter()
            .find(|item| item.kind == TypeScriptSemanticSymbolKind::LocaleEnum)
            .expect("locale enum");
        let function = artifacts
            .iter()
            .find(|item| item.kind == TypeScriptSemanticSymbolKind::LocaleFunction)
            .expect("function");
        assert!(function
            .imports
            .iter()
            .any(|dependency| { dependency.name == locale_enum.name && dependency.type_only }));
        let message = project.message_artifacts().expect("messages").remove(0);
        let physical = compile_typescript_bundler_message_module(
            &project,
            "en",
            "root",
            &message.output_file_name,
            &message.shared_import_path,
            &message.runtime_import_path,
            &sources(schema, locale),
        )
        .expect("physical");
        assert!(physical
            .code
            .contains(&format!("import {{ {} }}", form.binding)));
        assert!(!physical.code.contains(&format!("const {} =", form.binding)));
        let enum_module = compile_typescript_bundler_semantic_module(
            &project,
            locale_enum,
            &sources(schema, locale),
        )
        .expect("enum module");
        assert!(enum_module.code.contains("export type Gender"));
        assert_eq!(enum_module.code.matches("export ").count(), 1);
    }

    #[test]
    fn inline_binding_preserves_parameter_type_for_form_dependencies() {
        let schema = "enum Fruit { apple }\nroot(fruit: Fruit)\n";
        let locale = "impl Fruit { apple { label = Apple } }\nroot = {fn(alias: fruit) { _ => {alias.label} }}\n";
        let project = project(schema, locale);
        let form = project
            .semantic_artifacts()
            .expect("artifacts")
            .into_iter()
            .find(|item| item.kind == TypeScriptSemanticSymbolKind::LocaleForm)
            .expect("form");
        let message = project.message_artifacts().expect("messages").remove(0);

        assert!(message
            .semantic_imports
            .iter()
            .any(|dependency| dependency.module_path == form.module_path));
        let compiled = compile_typescript_bundler_message_artifact_module(
            &project,
            &message,
            &sources(schema, locale),
        )
        .expect("message");
        assert!(compiled
            .code
            .contains(&format!("import {{ {} }}", form.binding)));
        assert!(!compiled.code.contains(&format!("const {} =", form.binding)));
    }

    #[test]
    fn literal_symbol_names_do_not_create_false_dependencies() {
        let schema = "root\n";
        let locale = "let unused = ignored\nroot = unused selectBranch formatNumber\n";
        let project = project(schema, locale);
        assert!(project.semantic_artifacts().expect("artifacts").is_empty());
        assert!(project.message_artifacts().expect("messages")[0]
            .semantic_imports
            .is_empty());
    }

    #[test]
    fn long_names_are_portable_and_semantic_cycles_are_rejected() {
        let long_name = "a".repeat(400);
        let path = semantic_module_path(
            "en",
            TypeScriptSemanticSymbolKind::LocaleVariable,
            &long_name,
        )
        .expect("portable path");
        assert!(path.len() <= MAX_PORTABLE_PATH_BYTES);
        assert!(path.is_ascii());
        assert_eq!(
            path,
            semantic_module_path(
                "en",
                TypeScriptSemanticSymbolKind::LocaleVariable,
                &long_name
            )
            .unwrap()
        );

        let make = |name: &str, dependency: &str| TypeScriptSemanticArtifact {
            locale: "en".into(),
            kind: TypeScriptSemanticSymbolKind::LocaleVariable,
            name: name.into(),
            binding: name.into(),
            module_path: format!("{name}.ts"),
            source_map_path: format!("{name}.ts.map"),
            output_file_name: format!("{name}.ts"),
            shared_import_path: "./shared".into(),
            runtime_module_path: "locales/en/_runtime.ts".into(),
            runtime_import_path: "./runtime".into(),
            source_ids: vec![],
            imports: vec![TypeScriptSemanticImport {
                kind: TypeScriptSemanticSymbolKind::LocaleVariable,
                name: dependency.into(),
                binding: dependency.into(),
                module_path: format!("{dependency}.ts"),
                import_path: format!("./{dependency}"),
                source_ids: vec![],
                type_only: false,
            }],
        };
        assert!(matches!(
            validate_cycles(&[make("a", "b"), make("b", "a")]),
            Err(TypeScriptCodegenError::SemanticDependencyCycle { .. })
        ));
    }

    #[test]
    fn semantic_source_ids_include_nested_values_from_other_sources() {
        let mut locale = lower_locale(
            &parse_locale_in("impl Fruit { apple { label = Apple } }\n", SourceId(20))
                .expect("locale"),
        );
        let IrFormEntry::Attribute { value, .. } = &mut locale.forms[0].variants[0].entries[0]
        else {
            panic!("attribute")
        };
        let IrValue::Text(text) = value else {
            panic!("text")
        };
        text.span.source = SourceId(21);

        assert_eq!(
            symbol_source_ids(&locale, TypeScriptSemanticSymbolKind::LocaleForm, "Fruit"),
            vec![SourceId(20), SourceId(21)]
        );
    }
}
