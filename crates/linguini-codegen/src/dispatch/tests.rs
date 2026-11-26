use super::*;
use crate::DecisionArm;
use linguini_ir::{lower_locale_typed, IrTextPart};
use linguini_syntax::parse_locale;

fn literal(text: &IrText) -> String {
    text.parts
        .iter()
        .map(|part| match part {
            IrTextPart::Text(value) => value.as_str(),
            IrTextPart::Placeholder(_) => panic!("fixture must contain literal text"),
        })
        .collect()
}

#[test]
fn nested_function_dimensions_keep_full_indices_and_merge_projected_results() {
    let locale = lower_locale_typed(
        &parse_locale(concat!(
            "fn Label(Plural, Choice, payload: String) {\n",
            " one {\n a => SAME\n b => same\n other => Other\n _ => Default\n }\n",
            " _ => Outer\n}\n",
        ))
        .unwrap(),
    );
    let function = &locale.functions()[0];
    // The planner identifies dimensions independently of a backend's parameter layout.
    let parameters = [
        function.parameters[2].clone(),
        function.parameters[0].clone(),
        function.parameters[2].clone(),
        function.parameters[1].clone(),
    ];
    let mut visited = Vec::new();
    let mut projected = Vec::new();
    let plan = function_dispatch_plan(
        &parameters,
        &function.branches,
        |index| {
            visited.push(index);
            index
        },
        |text| {
            let value = literal(text);
            projected.push(value.clone());
            value.to_lowercase()
        },
    );
    assert_eq!(visited, [Some(1), Some(3)]);
    assert_eq!(projected, ["SAME", "same", "Other", "Default", "Outer"]);
    assert_eq!(
        plan,
        Decision::Branch {
            selector: Some(1),
            arms: vec![DecisionArm {
                keys: vec!["one".into()],
                result: Decision::Branch {
                    selector: Some(3),
                    arms: vec![
                        DecisionArm {
                            keys: vec!["a".into(), "b".into()],
                            result: Decision::Return("same".into()),
                        },
                        DecisionArm {
                            keys: vec!["other".into()],
                            result: Decision::Return("other".into()),
                        },
                    ],
                    fallback: Some(Box::new(Decision::Return("default".into()))),
                },
            }],
            fallback: Some(Box::new(Decision::Return("outer".into()))),
        }
    );
}

#[test]
fn absent_dimensions_select_undefined_then_default_then_other_without_projecting_dead_bodies() {
    let locale = lower_locale_typed(&parse_locale("fn Ready() {\n _ => ready\n}\n").unwrap());
    let function = &locale.functions()[0];
    assert_eq!(
        function_dispatch_plan(
            &function.parameters,
            &function.branches,
            |_| panic!("a zero-input body needs no selector"),
            literal,
        ),
        Decision::<(), _>::Return("ready".into())
    );
    let mut branches = ["other", "_", "undefined"]
        .into_iter()
        .map(|key| {
            let mut branch = function.branches[0].clone();
            branch.key = key.into();
            let IrFunctionBranchValue::Text(text) = &mut branch.value else {
                unreachable!()
            };
            text.parts = vec![IrTextPart::Text(key.into())];
            branch
        })
        .collect::<Vec<_>>();
    for expected in ["undefined", "_", "other"] {
        assert_eq!(
            function_dispatch_plan(
                &[],
                &branches,
                |_| panic!("a resolved missing selector must not be projected"),
                |text| {
                    let value = literal(text);
                    assert_eq!(value, expected, "only the selected body is projected");
                    value
                },
            ),
            Decision::<(), _>::Return(expected.into())
        );
        branches.pop();
    }
    assert_eq!(
        function_dispatch_plan(&[], &[], |index| index, literal),
        Decision::Branch {
            selector: None,
            arms: vec![],
            fallback: None,
        }
    );
}

#[test]
fn form_aliases_project_once_and_keep_selector_even_when_all_results_merge() {
    let locale = lower_locale_typed(&parse_locale("fn Ready() {\n _ => ready\n}\n").unwrap());
    let function = &locale.functions()[0];
    let IrFunctionBranchValue::Text(text) = &function.branches[0].value else {
        unreachable!()
    };
    let mut branches = vec![
        IrBranch {
            keys: vec!["one".into(), "few".into()],
            value: text.clone(),
            span: text.span,
        },
        IrBranch {
            keys: vec![],
            value: text.clone(),
            span: text.span,
        },
    ];
    branches[1].value.parts = vec![IrTextPart::Text("fallback".into())];
    let mut projected = 0;
    let plan = form_dispatch_plan(&branches, "selector()", |text| {
        projected += 1;
        literal(text)
    });
    assert_eq!(projected, 2, "multiple keys share one projected body");
    assert_eq!(
        plan,
        Decision::Branch {
            selector: "selector()",
            arms: vec![DecisionArm {
                keys: vec!["one".into(), "few".into()],
                result: Decision::Return("ready".into()),
            }],
            fallback: Some(Box::new(Decision::Return("fallback".into()))),
        }
    );
    branches[1].value = text.clone();
    assert_eq!(
        form_dispatch_plan(&branches, "mayThrow()", literal),
        Decision::Branch {
            selector: "mayThrow()",
            arms: vec![],
            fallback: Some(Box::new(Decision::Return("ready".into()))),
        }
    );
    let mut branch = function.branches[0].clone();
    branch.key = "other".into();
    assert_eq!(
        function_dispatch_plan(
            &[IrFunctionParameter {
                name: None,
                ty: "Plural".into()
            }],
            &[branch],
            |index| {
                assert_eq!(index, Some(0));
                "mayThrow()"
            },
            literal,
        ),
        Decision::Branch {
            selector: "mayThrow()",
            arms: vec![],
            fallback: Some(Box::new(Decision::Return("ready".into()))),
        }
    );
}
