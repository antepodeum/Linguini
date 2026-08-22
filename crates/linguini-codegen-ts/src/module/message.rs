//! Public compiler for one bundler-native TypeScript message module.

use std::collections::{BTreeMap, BTreeSet};

use linguini_cldr::canonicalize_locale;
use linguini_ir::{IrForm, IrFormEntry, IrFunction, IrModule, IrSymbolKind, IrValue};
use linguini_syntax::{SourceId, Span};

use crate::ecmascript::{
    EcmaImport, EcmaImportBindings, EcmaModule, EcmaNamedImport, EcmaSource, EcmaStatement,
};

use super::deps::MessageDependencyClosure;
use super::emit::{self, emit_formatter_data, emit_forms, emit_local_functions, emit_variables};
use super::formatters::{formatter_requirements, plural_required};
use super::names::emit_docs;
use super::semantic::TypeScriptSemanticImport;
use super::signature::MessageCallSignature;
use super::{
    TypeScriptCodegenError, TypeScriptMessageArtifact, TypeScriptOptions,
    ValidatedTypeScriptProject,
};

/// One complete physical ESM module for one schema message and locale.
///
/// The generated module has exactly one public export, `message`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTypeScriptMessageModule {
    pub locale: String,
    pub message: String,
    pub code: String,
    pub source_map: String,
    pub source_ids: Vec<SourceId>,
}

impl CompiledTypeScriptMessageModule {
    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn source_map(&self) -> &str {
        &self.source_map
    }

    pub fn source_ids(&self) -> &[SourceId] {
        &self.source_ids
    }
}

/// Compiles one validated project message into a deterministic source-mapped TypeScript ESM
/// module. The closure is computed from the validated project, so callers cannot accidentally
/// emit unvalidated IR or pull unrelated project leaves into the output.
pub fn compile_typescript_message_module(
    project: &ValidatedTypeScriptProject<'_>,
    locale: &str,
    canonical_message: &str,
    output_file_name: &str,
    shared_import_path: &str,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptMessageModule, TypeScriptCodegenError> {
    compile_message_module(
        project,
        MessageModuleRequest {
            locale,
            canonical_message,
            output_file_name,
            shared_import_path,
            emission: MessageModuleEmission::Standalone,
            runtime_import_path: None,
            semantic_imports: None,
        },
        sources,
    )
}

/// Compiles one physical bundler leaf which shares locale-specific formatter and plural helpers.
///
/// `runtime_import_path` is an ESM specifier from the generated message module to the effective
/// locale runtime emitted by [`super::generate_typescript_project_files`].
pub fn compile_typescript_bundler_message_module(
    project: &ValidatedTypeScriptProject<'_>,
    locale: &str,
    canonical_message: &str,
    output_file_name: &str,
    shared_import_path: &str,
    runtime_import_path: &str,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptMessageModule, TypeScriptCodegenError> {
    compile_message_module(
        project,
        MessageModuleRequest {
            locale,
            canonical_message,
            output_file_name,
            shared_import_path,
            emission: MessageModuleEmission::Bundler,
            runtime_import_path: Some(runtime_import_path),
            semantic_imports: None,
        },
        sources,
    )
}

/// Compiles one canonical physical message artifact without recomputing or inferring its paths.
pub fn compile_typescript_bundler_message_artifact_module(
    project: &ValidatedTypeScriptProject<'_>,
    artifact: &TypeScriptMessageArtifact,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptMessageModule, TypeScriptCodegenError> {
    compile_message_module(
        project,
        MessageModuleRequest {
            locale: &artifact.locale,
            canonical_message: &artifact.message,
            output_file_name: &artifact.output_file_name,
            shared_import_path: &artifact.shared_import_path,
            emission: MessageModuleEmission::Bundler,
            runtime_import_path: Some(&artifact.runtime_import_path),
            semantic_imports: Some(&artifact.semantic_imports),
        },
        sources,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageModuleEmission {
    Standalone,
    Bundler,
}

struct MessageModuleRequest<'a> {
    locale: &'a str,
    canonical_message: &'a str,
    output_file_name: &'a str,
    shared_import_path: &'a str,
    emission: MessageModuleEmission,
    runtime_import_path: Option<&'a str>,
    semantic_imports: Option<&'a [TypeScriptSemanticImport]>,
}

fn compile_message_module(
    project: &ValidatedTypeScriptProject<'_>,
    request: MessageModuleRequest<'_>,
    sources: &[EcmaSource],
) -> Result<CompiledTypeScriptMessageModule, TypeScriptCodegenError> {
    let MessageModuleRequest {
        locale,
        canonical_message,
        output_file_name,
        shared_import_path,
        emission,
        runtime_import_path,
        semantic_imports,
    } = request;
    let requested_locale = canonicalize_locale(locale).unwrap_or_else(|_| locale.to_owned());
    let project_locale = project
        .locales
        .iter()
        .find(|entry| {
            entry.locale == locale
                || entry.locale.eq_ignore_ascii_case(locale)
                || canonicalize_locale(&entry.locale)
                    .map(|canonical| canonical.eq_ignore_ascii_case(&requested_locale))
                    .unwrap_or(false)
        })
        .map(|entry| entry.locale.as_str())
        .ok_or_else(|| TypeScriptCodegenError::UnknownLocale {
            locale: locale.to_owned(),
        })?;
    let canonical_locale =
        canonicalize_locale(project_locale).unwrap_or_else(|_| project_locale.to_owned());

    let closure = project.message_dependency_closure(project_locale, canonical_message)?;
    let source_ids = closure.source_ids().to_vec();
    let source_records = ordered_sources(&source_ids, sources)?;
    let options = super::project_locale_options(&canonical_locale, &project.options)?;
    let semantic_imports = if runtime_import_path.is_some() {
        match semantic_imports {
            Some(imports) => imports.to_vec(),
            None => super::semantic::message_imports_from_shared(
                project,
                project_locale,
                canonical_message,
                shared_import_path,
            )?,
        }
    } else {
        Vec::new()
    };
    let module = emit_message_module(
        &closure,
        &options,
        shared_import_path,
        emission,
        runtime_import_path,
        &semantic_imports,
    );
    let rendered = module.render(output_file_name, &source_records);

    Ok(CompiledTypeScriptMessageModule {
        locale: canonical_locale,
        message: canonical_message.to_owned(),
        code: rendered.code,
        source_map: rendered.source_map,
        source_ids,
    })
}

fn ordered_sources(
    source_ids: &[SourceId],
    sources: &[EcmaSource],
) -> Result<Vec<EcmaSource>, TypeScriptCodegenError> {
    let mut seen = BTreeSet::new();
    for source in sources {
        if !seen.insert(source.id) {
            return Err(TypeScriptCodegenError::DuplicateMessageSource {
                source_id: source.id,
            });
        }
    }
    let records = sources
        .iter()
        .map(|source| (source.id, source))
        .collect::<BTreeMap<_, _>>();
    source_ids
        .iter()
        .map(|source_id| {
            records
                .get(source_id)
                .map(|source| (*source).clone())
                .ok_or(TypeScriptCodegenError::MissingMessageSource {
                    source_id: *source_id,
                })
        })
        .collect()
}

fn emit_message_module(
    closure: &MessageDependencyClosure,
    options: &TypeScriptOptions,
    shared_import_path: &str,
    emission: MessageModuleEmission,
    runtime_import_path: Option<&str>,
    semantic_imports: &[TypeScriptSemanticImport],
) -> EcmaModule {
    let schema = closure.schema();
    let complete_locale = closure.locale_module();
    let mut physical_locale;
    let locale = if runtime_import_path.is_some() {
        physical_locale = complete_locale.clone();
        physical_locale.enums.clear();
        physical_locale.variables.clear();
        physical_locale.forms.clear();
        physical_locale.functions.clear();
        &physical_locale
    } else {
        complete_locale
    };
    let mut statements = Vec::new();

    if runtime_import_path.is_none() {
        let mut formatter_data = String::new();
        emit_formatter_data(schema, locale, options, &mut formatter_data);
        push_chunk(&mut statements, formatter_data, None);
    }

    for item in &locale.enums {
        let mut one = IrModule::default();
        one.enums.push(item.clone());
        let mut output = String::new();
        emit::emit_locale_enum_types(schema, &one, &mut output);
        push_chunk(
            &mut statements,
            output,
            symbol_span(locale, IrSymbolKind::Enum, &item.name, None),
        );
    }

    for item in &locale.variables {
        let mut one = IrModule::default();
        one.variables.push(item.clone());
        let mut output = String::new();
        emit_variables(&one, options, &mut output);
        push_chunk(
            &mut statements,
            output,
            symbol_span(
                locale,
                IrSymbolKind::Variable,
                &item.name,
                Some(item.value.span),
            ),
        );
    }

    for item in &locale.forms {
        let mut one = IrModule::default();
        one.forms.push(item.clone());
        let mut output = String::new();
        emit_forms(&one, options, &mut output);
        push_chunk(
            &mut statements,
            output,
            symbol_span(locale, IrSymbolKind::Form, &item.name, form_span(item)),
        );
    }

    for item in &locale.functions {
        let mut one = IrModule::default();
        one.functions.push(item.clone());
        let mut output = String::new();
        emit_local_functions(&one, options, &mut output);
        push_chunk(
            &mut statements,
            output,
            symbol_span(
                locale,
                IrSymbolKind::Function,
                &item.name,
                function_span(item),
            ),
        );
    }

    if let (Some(signature), Some(implementation)) = (
        schema
            .messages
            .iter()
            .find(|item| item.name == closure.message),
        locale
            .messages
            .iter()
            .find(|item| item.name == closure.message),
    ) {
        let call_signature = MessageCallSignature::from_message(signature);
        let body = emit::message_body(schema, signature, implementation, options);
        let output = if !call_signature.is_parameterized() {
            let mut docs = String::new();
            emit_docs(&signature.docs, "", &mut docs);
            match emission {
                MessageModuleEmission::Standalone => {
                    format!("{docs}export const message = (): string => {body};\n")
                }
                MessageModuleEmission::Bundler => {
                    format!("{docs}export const message = {body};\n")
                }
            }
        } else {
            format!(
                "{}export function message({}): string {{\n  {}\n  return {body};\n}}\n",
                call_signature.implementation_overloads("message", &signature.docs),
                call_signature.implementation_rest_params(),
                call_signature.normalized_bindings_statement()
            )
        };
        push_chunk(
            &mut statements,
            output,
            message_span(schema, locale, signature, implementation),
        );
    }

    let uses_plural = plural_required(schema, locale);
    let uses_select_branch = statements
        .iter()
        .any(|(code, _)| code.contains("selectBranch("));
    let uses_named_message_args = schema
        .messages
        .iter()
        .any(|signature| signature.name == closure.message && !signature.parameters.is_empty());
    let runtime_helpers =
        runtime_import_path.map(|_| formatter_requirements(schema, locale).helper_names());
    let mut module = EcmaModule {
        imports: message_imports(MessageImportRequest {
            shared_import_path,
            runtime_import_path,
            schema,
            plural_function: &options.plural_function,
            uses_select_branch,
            uses_named_message_args,
            uses_plural,
            runtime_helpers: runtime_helpers.as_deref().unwrap_or_default(),
        }),
        statements: Vec::new(),
    };
    for dependency in semantic_imports {
        module.imports.push(EcmaImport {
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
        });
    }
    if uses_plural && runtime_import_path.is_none() {
        let mut plural_helpers = String::new();
        emit::emit_plural_helpers(options, &mut plural_helpers);
        push_statement(&mut module, plural_helpers, None);
    }
    for (code, span) in statements {
        push_statement(&mut module, code, span);
    }
    module
}

struct MessageImportRequest<'a> {
    shared_import_path: &'a str,
    runtime_import_path: Option<&'a str>,
    schema: &'a IrModule,
    plural_function: &'a str,
    uses_select_branch: bool,
    uses_named_message_args: bool,
    uses_plural: bool,
    runtime_helpers: &'a [&'a str],
}

fn message_imports(request: MessageImportRequest<'_>) -> Vec<EcmaImport> {
    let mut imports = Vec::new();
    let type_names = emit::schema_type_names(request.schema);
    if !type_names.is_empty() {
        imports.push(EcmaImport {
            specifier: request.shared_import_path.to_owned(),
            bindings: EcmaImportBindings::TypeNamed(
                type_names
                    .iter()
                    .map(|name| EcmaNamedImport::new(name, name))
                    .collect(),
            ),
        });
    }
    let mut shared_helpers = Vec::new();
    if request.uses_select_branch {
        shared_helpers.push("selectBranch");
    }
    if request.uses_named_message_args {
        shared_helpers.push("normalizeMessageArgs");
    }
    if !shared_helpers.is_empty() {
        imports.push(EcmaImport::named(
            request.shared_import_path,
            shared_helpers
                .into_iter()
                .map(|name| EcmaNamedImport::new(name, name))
                .collect(),
        ));
    }
    if let Some(runtime_import_path) = request.runtime_import_path {
        let mut helpers = request.runtime_helpers.to_vec();
        if request.uses_plural {
            helpers.push(request.plural_function);
        }
        if !helpers.is_empty() {
            imports.push(EcmaImport::named(
                runtime_import_path,
                helpers
                    .into_iter()
                    .map(|name| EcmaNamedImport::new(name, name))
                    .collect(),
            ));
        }
    }
    imports
}

fn push_chunk(chunks: &mut Vec<(String, Option<Span>)>, code: String, span: Option<Span>) {
    if !code.trim().is_empty() {
        chunks.push((code, span));
    }
}

fn push_statement(module: &mut EcmaModule, code: String, span: Option<Span>) {
    if code.trim().is_empty() {
        return;
    }
    module.statements.push(match span {
        Some(span) => EcmaStatement::mapped(code, span),
        None => EcmaStatement::generated(code),
    });
}

fn symbol_span(
    module: &IrModule,
    kind: IrSymbolKind,
    name: &str,
    fallback: Option<Span>,
) -> Option<Span> {
    module
        .origins
        .iter()
        .rev()
        .find(|origin| origin.kind == kind && origin.name == name)
        .map(|origin| origin.span)
        .or(fallback)
}

fn message_span(
    schema: &IrModule,
    locale: &IrModule,
    signature: &linguini_ir::IrMessage,
    implementation: &linguini_ir::IrMessage,
) -> Option<Span> {
    implementation
        .body
        .as_ref()
        .map(|body| body.span)
        .or_else(|| signature.body.as_ref().map(|body| body.span))
        .or_else(|| symbol_span(locale, IrSymbolKind::Message, &implementation.name, None))
        .or_else(|| symbol_span(schema, IrSymbolKind::Message, &signature.name, None))
}

fn function_span(function: &IrFunction) -> Option<Span> {
    function.branches.first().map(|branch| branch.span)
}

fn form_span(form: &IrForm) -> Option<Span> {
    form.variants
        .iter()
        .flat_map(|variant| variant.entries.iter())
        .find_map(form_entry_span)
}

fn form_entry_span(entry: &IrFormEntry) -> Option<Span> {
    match entry {
        IrFormEntry::Attribute { value, .. } => value_span(value),
        IrFormEntry::Branch(branch) => Some(branch.span),
    }
}

fn value_span(value: &IrValue) -> Option<Span> {
    match value {
        IrValue::Text(text) => Some(text.span),
        IrValue::Map(branches) => branches.first().map(|branch| branch.span),
        IrValue::Object(entries) => entries.iter().find_map(form_entry_span),
    }
}

#[cfg(test)]
mod tests {
    use super::{compile_typescript_bundler_message_module, compile_typescript_message_module};
    use crate::{
        EcmaSource, TypeScriptLocaleModule, TypeScriptProjectOptions, ValidatedTypeScriptProject,
    };
    use linguini_ir::{lower_locale, lower_schema};
    use linguini_syntax::{parse_locale_in, parse_schema_in, SourceId};

    const SCHEMA: &str = "/// Greeting\nroot(name: String)\ndrop\n";
    const LOCALE: &str = "let helper = Hello\nroot = {helper}, {name}\ndrop = Drop\n";

    fn project() -> ValidatedTypeScriptProject<'static> {
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in(SCHEMA, SourceId(7)).expect("schema"),
        )));
        let locale = lower_locale(&parse_locale_in(LOCALE, SourceId(8)).expect("locale"));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }]));
        let options = TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        ValidatedTypeScriptProject::try_new(schema, locales, &options).expect("project")
    }

    #[test]
    fn emits_one_callable_export_and_sorted_sources() {
        let result = compile_typescript_message_module(
            &project(),
            "en",
            "root",
            "messages/root.ts",
            "../../shared",
            &[
                EcmaSource::new(SourceId(8), "locale.lgs", LOCALE),
                EcmaSource::new(SourceId(7), "schema.lgs", SCHEMA),
            ],
        )
        .expect("compile");

        assert_eq!(result.locale, "en");
        assert_eq!(result.message, "root");
        assert_eq!(result.source_ids, vec![SourceId(7), SourceId(8)]);
        assert!(result
            .code
            .contains("export function message(name: string)"));
        assert!(result
            .code
            .contains("export function message(args: { name: string }): string;"));
        assert_eq!(result.code.matches("export function message").count(), 3);
        assert_eq!(result.code.matches("return String(helper)").count(), 1);
        assert!(result
            .code
            .contains("normalizeMessageArgs(__lgl_args, [\"name\"])"));
        assert!(result.code.ends_with("//# sourceMappingURL=root.ts.map\n"));
        assert!(result
            .source_map
            .contains("\"sources\":[\"schema.lgs\",\"locale.lgs\"]"));
        assert!(result
            .source_map
            .split("\"mappings\":\"")
            .nth(1)
            .is_some_and(|mapping| !mapping.starts_with('"')));
        assert!(!result.code.contains("drop"));

        let reordered = compile_typescript_message_module(
            &project(),
            "en",
            "root",
            "messages/root.ts",
            "../../shared",
            &[
                EcmaSource::new(SourceId(7), "schema.lgs", SCHEMA),
                EcmaSource::new(SourceId(8), "locale.lgs", LOCALE),
            ],
        )
        .expect("compile");
        assert_eq!(result.code, reordered.code);
        assert_eq!(result.source_map, reordered.source_map);
    }

    #[test]
    fn parameterless_message_emission_is_explicit() {
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in("/// Greeting\nroot\n", SourceId(7)).expect("schema"),
        )));
        let locale = lower_locale(&parse_locale_in("root = Hello\n", SourceId(8)).expect("locale"));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }]));
        let options = TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        let project = ValidatedTypeScriptProject::try_new(schema, locales, &options).unwrap();
        let result = compile_typescript_message_module(
            &project,
            "en",
            "root",
            "root.ts",
            "./shared",
            &[
                EcmaSource::new(SourceId(7), "schema.lgs", "/// Greeting\nroot\n"),
                EcmaSource::new(SourceId(8), "locale.lgs", "root = Hello\n"),
            ],
        )
        .unwrap();
        assert!(result.code.contains("export const message = (): string =>"));
        assert_eq!(result.code.matches("export ").count(), 1);
        assert!(!result.code.contains("plural("));

        let bundler = compile_typescript_bundler_message_module(
            &project,
            "en",
            "root",
            "root.ts",
            "./shared",
            "./runtime",
            &[
                EcmaSource::new(SourceId(8), "locale.lgs", "root = Hello\n"),
                EcmaSource::new(SourceId(7), "schema.lgs", "/// Greeting\nroot\n"),
            ],
        )
        .unwrap();
        assert!(bundler.code.contains("export const message = \"Hello\";"));
        assert!(!bundler.code.contains("message = (): string =>"));
        assert_eq!(bundler.code.matches("export ").count(), 1);
        let reordered_bundler = compile_typescript_bundler_message_module(
            &project,
            "en",
            "root",
            "root.ts",
            "./shared",
            "./runtime",
            &[
                EcmaSource::new(SourceId(7), "schema.lgs", "/// Greeting\nroot\n"),
                EcmaSource::new(SourceId(8), "locale.lgs", "root = Hello\n"),
            ],
        )
        .unwrap();
        assert_eq!(bundler.code, reordered_bundler.code);
        assert_eq!(bundler.source_map, reordered_bundler.source_map);
    }

    #[test]
    fn missing_dependency_source_is_precise() {
        let error = compile_typescript_message_module(
            &project(),
            "en",
            "root",
            "root.ts",
            "./shared",
            &[EcmaSource::new(SourceId(7), "schema.lgs", "")],
        )
        .expect_err("missing source should fail");
        assert!(matches!(
            error,
            crate::TypeScriptCodegenError::MissingMessageSource {
                source_id: SourceId(8)
            }
        ));
    }

    #[test]
    fn duplicate_source_records_are_rejected_before_indexing() {
        let error = compile_typescript_message_module(
            &project(),
            "en",
            "root",
            "root.ts",
            "./shared",
            &[
                EcmaSource::new(SourceId(7), "schema.lgs", SCHEMA),
                EcmaSource::new(SourceId(7), "schema-copy.lgs", SCHEMA),
                EcmaSource::new(SourceId(8), "locale.lgs", LOCALE),
            ],
        )
        .expect_err("duplicate source should fail");
        assert!(matches!(
            error,
            crate::TypeScriptCodegenError::DuplicateMessageSource {
                source_id: SourceId(7)
            }
        ));
    }

    #[test]
    fn plural_and_dispatch_helpers_are_included_only_when_referenced() {
        let schema_text = "summary(count: Number)\n";
        let locale_text = "summary = {fn(Plural(count)) {\n  one => One\n  other => Many\n}}\n";
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in(schema_text, SourceId(17)).expect("schema"),
        )));
        let locale = lower_locale(&parse_locale_in(locale_text, SourceId(18)).expect("locale"));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }]));
        let options = TypeScriptProjectOptions {
            base_locale: Some("en".to_owned()),
            ..TypeScriptProjectOptions::default()
        };
        let project = ValidatedTypeScriptProject::try_new(schema, locales, &options).unwrap();
        let result = compile_typescript_message_module(
            &project,
            "en",
            "summary",
            "summary.ts",
            "./shared",
            &[
                EcmaSource::new(SourceId(17), "schema.lgs", schema_text),
                EcmaSource::new(SourceId(18), "locale.lgs", locale_text),
            ],
        )
        .unwrap();
        assert!(result.code.contains("function pluralEn("));
        assert!(result
            .code
            .contains("import { selectBranch, normalizeMessageArgs }"));
        assert!(result.code.contains("pluralEn(count)"));
    }

    #[test]
    fn bundler_leaf_imports_exact_runtime_helpers_without_helper_bodies() {
        let schema_text = "summary(count: Number)\n";
        let locale_text =
            "summary = {fn(Plural(count)) {\n  one => {count @number}\n  other => Many\n}}\n";
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in(schema_text, SourceId(27)).expect("schema"),
        )));
        let locale = lower_locale(&parse_locale_in(locale_text, SourceId(28)).expect("locale"));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }]));
        let project = ValidatedTypeScriptProject::try_new(
            schema,
            locales,
            &TypeScriptProjectOptions {
                base_locale: Some("en".to_owned()),
                ..TypeScriptProjectOptions::default()
            },
        )
        .unwrap();
        let sources = [
            EcmaSource::new(SourceId(27), "schema.lgs", schema_text),
            EcmaSource::new(SourceId(28), "locale.lgl", locale_text),
        ];

        let shared = compile_typescript_bundler_message_module(
            &project,
            "en",
            "summary",
            "summary.ts",
            "../../../shared",
            "../../../locales/en/_runtime",
            &sources,
        )
        .unwrap();
        assert!(shared
            .code
            .contains("import { formatNumber, pluralEn } from \"../../../locales/en/_runtime\";"));
        assert!(shared
            .code
            .contains("import { selectBranch, normalizeMessageArgs } from \"../../../shared\";"));
        assert!(!shared.code.contains("function formatNumber("));
        assert!(!shared.code.contains("function pluralEn("));

        let standalone = compile_typescript_message_module(
            &project,
            "en",
            "summary",
            "summary.ts",
            "../../../shared",
            &sources,
        )
        .unwrap();
        assert!(standalone.code.contains("function formatNumber("));
        assert!(standalone.code.contains("function pluralEn("));
        assert!(!standalone.code.contains("_runtime"));
    }

    #[test]
    fn literal_helper_names_do_not_create_bundler_runtime_imports() {
        let schema_text = "literal\n";
        let locale_text = "literal = formatNumber( pluralEn(\n";
        let schema = Box::leak(Box::new(lower_schema(
            &parse_schema_in(schema_text, SourceId(37)).expect("schema"),
        )));
        let locale = lower_locale(&parse_locale_in(locale_text, SourceId(38)).expect("locale"));
        let locales = Box::leak(Box::new(vec![TypeScriptLocaleModule {
            locale: "en".to_owned(),
            module: locale,
        }]));
        let project = ValidatedTypeScriptProject::try_new(
            schema,
            locales,
            &TypeScriptProjectOptions {
                base_locale: Some("en".to_owned()),
                ..TypeScriptProjectOptions::default()
            },
        )
        .unwrap();
        let compiled = compile_typescript_bundler_message_module(
            &project,
            "en",
            "literal",
            "literal.ts",
            "../../../shared",
            "../../../locales/en/_runtime",
            &[
                EcmaSource::new(SourceId(37), "schema.lgs", schema_text),
                EcmaSource::new(SourceId(38), "locale.lgl", locale_text),
            ],
        )
        .unwrap();

        assert!(compiled.code.contains("formatNumber( pluralEn("));
        assert!(!compiled
            .code
            .contains("from \"../../../locales/en/_runtime\""));
        assert!(!compiled.code.contains("function pluralEn("));
    }
}
