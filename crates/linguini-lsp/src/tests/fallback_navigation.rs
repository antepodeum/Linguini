use super::*;

fn locale(tag: &str, text: &str, fallbacks: &[&str]) -> LinguiniDocument {
    LinguiniDocument::new(format!("file:///main/{tag}.lgl"), "linguini-locale", text)
        .with_source_identity("main", Some(tag.into()))
        .with_locale_fallbacks(fallbacks.iter().map(|tag| (*tag).to_owned()))
}

#[test]
fn inherited_values_and_calls_keep_the_selected_declaration_owner() {
    // Deliberately share default SourceId(0): ownership must retain the source, not guess
    // from numeric IDs or overlapping declaration spans.
    let base = locale("en", "let suffix = BASE\nfn render(count: Number) { _ => {count}|{suffix} }\nsummary = {render(count)}|{suffix}\n", &[]);
    let parent = locale(
        "en-001",
        "fn render(count: Number) { _ => PARENT {count}|{suffix} }\n",
        &["en"],
    );
    let child = locale(
        "en-AU",
        "summary = {render(count)}|{suffix}\n",
        &["en-001", "en"],
    );
    let sibling = locale("fr", "summary = {render(count)}|{suffix}\n", &["en"]);
    let local = locale("de", "let suffix = LOCAL\nfn render(count: Number) { _ => {count} }\nsummary = {render(count)}|{suffix}\n", &["en"]);
    let foreign =
        locale("it", &child.text, &["en"]).with_source_identity("account", Some("it".into()));
    let workspace = vec![
        base.clone(),
        parent.clone(),
        child.clone(),
        sibling.clone(),
        local.clone(),
        foreign,
    ];
    for (name, owner, expected_uris) in [
        ("render", &parent, vec![&parent.uri, &child.uri]),
        (
            "suffix",
            &base,
            vec![&base.uri, &parent.uri, &child.uri, &sibling.uri],
        ),
    ] {
        let offset = child.text.find(name).unwrap();
        let (uri, span) = definition_at_with_workspace(&child, offset, workspace.clone())
            .expect("inherited declaration");
        assert_eq!(uri, owner.uri);
        assert_eq!(span.start, owner.text.find(name).unwrap());
        let references = references_at_with_workspace(&child, offset, workspace.clone());
        let uris = references
            .iter()
            .map(|item| &item.uri)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(uris, expected_uris.into_iter().collect());
        assert_eq!(references.iter().filter(|item| item.declaration).count(), 1);
        let edits = rename_workspace_edits(workspace.clone(), &child, offset, "renamed");
        assert_eq!(edits.len(), references.len());
        for (edit, reference) in edits.iter().zip(&references) {
            assert_eq!(edit.uri, reference.uri);
            assert_eq!(edit.edit.span, reference.span);
        }
        let declaration = owner.text.find(name).unwrap();
        assert_eq!(
            references_at_with_workspace(owner, declaration, workspace.clone()),
            references
        );
    }
    let call = sibling.text.find("render").unwrap();
    assert_eq!(
        definition_at_with_workspace(&sibling, call, workspace.clone())
            .unwrap()
            .0,
        base.uri
    );
    let local_call = local.text.rfind("render").unwrap();
    assert_eq!(
        definition_at_with_workspace(&local, local_call, workspace)
            .unwrap()
            .0,
        local.uri
    );
}

#[test]
fn inherited_navigation_requires_explicit_unambiguous_fallbacks() {
    let base = locale(
        "en",
        "let suffix = BASE\nfn render(count: Number) { _ => {count} }\n",
        &[],
    );
    let source = locale("fr", "summary = {render(count)}|{suffix}\n", &["en"]);
    let mut duplicate = base.clone();
    duplicate.uri = "file:///duplicate/en.lgl".into();
    let foreign = base
        .clone()
        .with_source_identity("account", Some("en".into()));
    for (current, workspace) in [
        (source.clone().with_locale_fallbacks([]), vec![base.clone()]),
        (source.clone(), vec![base.clone(), duplicate]),
        (source.clone(), vec![foreign]),
    ] {
        for name in ["render", "suffix"] {
            let offset = current.text.rfind(name).unwrap();
            assert!(
                definition_at_with_workspace(&current, offset, workspace.clone()).is_none(),
                "{name}: {}",
                current.text
            );
        }
    }
    // A nearer message claims the whole name; any local message target must stay local.
    let current = locale(
        "fr",
        "render = LOCAL\nsuffix = LOCAL\nsummary = {render(count)}|{suffix}\n",
        &["en"],
    );
    for name in ["render", "suffix"] {
        let target = definition_at_with_workspace(
            &current,
            current.text.rfind(name).unwrap(),
            [base.clone()],
        );
        assert!(target.is_none_or(|(uri, _)| uri == current.uri));
    }
}

#[test]
fn inherited_calls_preserve_same_named_lexical_values() {
    let base = locale(
        "en",
        "fn helper(count: Number) { _ => {count} }\nlet suffix = BASE\n",
        &[],
    );
    let source = locale("fr", "fn Forward(helper: String, suffix: String, count: Number) { _ => {helper}|{suffix}|{helper(count)} }\nsummary = {fn(helper: value) { _ => {helper}|{helper(count)} }}\n", &["en"]);
    for (offset, _) in source.text.match_indices("helper(count)") {
        assert_eq!(
            definition_at_with_workspace(&source, offset, [base.clone()])
                .unwrap()
                .0,
            base.uri
        );
    }
    for name in ["helper", "suffix"] {
        let offset = source.text.find(&format!("{{{name}}}")).unwrap() + 1;
        let (uri, span) = definition_at_with_workspace(&source, offset, [base.clone()]).unwrap();
        assert_eq!(uri, source.uri);
        assert_eq!(span.start, source.text.find(name).unwrap());
    }
}

#[test]
fn inherited_locale_enum_references_follow_whole_declaration_ownership() {
    let base = locale(
        "en",
        "enum Tone { formal, casual }\nfn base(value: Tone) { _ => BASE }\n",
        &[],
    );
    let parent = locale("en-001", "enum Tone { quiet }\n", &["en"]);
    let child = locale(
        "en-AU",
        "fn paint(value: Tone) { _ => CHILD }\n",
        &["en-001", "en"],
    );
    let sibling = locale("fr", "fn paint(value: Tone) { _ => SIBLING }\n", &["en"]);
    let workspace = vec![base.clone(), parent.clone(), child.clone(), sibling.clone()];
    for (source, owner) in [(&child, &parent), (&sibling, &base)] {
        let offset = source.text.find("Tone").unwrap();
        let (uri, span) = definition_at_with_workspace(source, offset, workspace.clone()).unwrap();
        assert_eq!(uri, owner.uri);
        assert_eq!(span.start, owner.text.find("Tone").unwrap());
        let references = references_at_with_workspace(source, offset, workspace.clone());
        assert!(references
            .iter()
            .all(|item| item.uri == owner.uri || item.uri == source.uri));
        assert_eq!(references.iter().filter(|item| item.declaration).count(), 1);
    }
    let detached = child.clone().with_locale_fallbacks([]);
    let offset = detached.text.find("Tone").unwrap();
    assert!(definition_at_with_workspace(&detached, offset, workspace.clone()).is_none());
    assert!(
        references_at_with_workspace(&detached, offset, workspace.clone())
            .iter()
            .all(|item| item.uri == detached.uri)
    );
    assert!(
        rename_workspace_edits(workspace, &detached, offset, "Shade")
            .iter()
            .all(|item| item.uri == detached.uri)
    );
}

#[test]
fn inherited_rename_rejects_conflicts_and_navigation_prefers_the_current_buffer() {
    let base = locale("en", "let suffix = BASE\nlet occupied = OTHER\n", &[]);
    let saved = locale("fr", "summary = SAVED\n", &["en"]);
    let current = locale("fr", "summary = {suffix}\n", &["en"]);
    let offset = current.text.find("suffix").unwrap();
    let workspace = vec![base.clone(), saved];
    assert_eq!(
        definition_at_with_workspace(&current, offset, workspace.clone())
            .unwrap()
            .0,
        base.uri
    );
    assert!(rename_workspace_edits(workspace, &current, offset, "occupied").is_empty());
}

#[test]
fn rename_rejects_shared_inherited_bodies_with_different_global_bindings() {
    for (base_text, child_text, name) in [
        (
            "let label = BASE\nfn render() { _ => {label} }\n",
            "let label = LOCAL\nsummary = {render()}|{label}\n",
            "label",
        ),
        (
            "fn helper() { _ => BASE }\nfn render() { _ => {helper()} }\n",
            "fn helper() { _ => LOCAL }\nsummary = {render()}|{helper()}\n",
            "helper",
        ),
        (
            "enum Tone { formal }\nfn render(value: Tone) { _ => BASE }\n",
            "enum Tone { casual }\nsummary = LOCAL\n",
            "Tone",
        ),
    ] {
        let base = locale("en", base_text, &[]);
        let child = locale("fr", child_text, &["en"]);
        let workspace = vec![base.clone(), child.clone()];
        for source in [&base, &child] {
            let offset = source.text.find(name).unwrap();
            assert!(
                rename_workspace_edits(workspace.clone(), source, offset, "renamed").is_empty(),
                "{name} in {}",
                source.uri
            );
        }
        // The container itself can still be renamed across both sources.
        let offset = base.text.find("render").unwrap();
        assert!(!rename_workspace_edits(workspace, &base, offset, "renamed").is_empty());
    }
}

#[test]
fn unrelated_inherited_bodies_do_not_block_global_renames() {
    let base = locale(
        "en",
        "let label = BASE\nlet other = OTHER\nfn render() { _ => {other} }\n",
        &[],
    );
    let child = locale(
        "fr",
        "let label = LOCAL\nsummary = {render()}|{label}\n",
        &["en"],
    );
    let workspace = vec![base.clone(), child.clone()];
    for source in [&base, &child] {
        let offset = source.text.find("label").unwrap();
        let edits = rename_workspace_edits(workspace.clone(), source, offset, "renamed");
        assert_eq!(edits.len(), if source.uri == base.uri { 1 } else { 2 });
        assert!(edits.iter().all(|edit| edit.uri == source.uri));
    }
}
