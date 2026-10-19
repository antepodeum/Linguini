//! Static plans for allocation-free generated control flow.

/// A static decision tree. Runtime backends lower it to branches and returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision<Selector, Result> {
    Return(Result),
    Branch {
        selector: Selector,
        arms: Vec<DecisionArm<Selector, Result>>,
        fallback: Option<Box<Self>>,
    },
}

/// Keys with the same result share one emitted branch body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionArm<Selector, Result> {
    pub keys: Vec<String>,
    pub result: Decision<Selector, Result>,
}

impl<Selector: Clone + PartialEq, Result: Clone + PartialEq> Decision<Selector, Result> {
    /// Preserve `_` before `other` fallback precedence and merge identical results.
    /// The selector stays in the plan even when every branch has the same result:
    /// evaluating it may be observable or reject an invalid input.
    pub fn branch(selector: Selector, branches: Vec<(String, Self)>) -> Self {
        let fallback = branches
            .iter()
            .find(|(key, _)| key == "_")
            .or_else(|| branches.iter().find(|(key, _)| key == "other"))
            .map(|(_, result)| result);
        let mut arms: Vec<DecisionArm<Selector, Result>> = Vec::new();
        for (key, result) in &branches {
            if key == "_" || Some(result) == fallback {
                continue;
            }
            if let Some(arm) = arms.iter_mut().find(|arm| arm.result == *result) {
                arm.keys.push(key.clone());
            } else {
                arms.push(DecisionArm {
                    keys: vec![key.clone()],
                    result: result.clone(),
                });
            }
        }
        Self::Branch {
            selector,
            arms,
            fallback: fallback.map(|result| Box::new(result.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decision, DecisionArm};

    #[test]
    fn groups_results_preserves_fallback_precedence_and_selector_evaluation() {
        let leaf = Decision::Return;
        let planned = Decision::branch(
            "selector",
            vec![
                ("male".into(), leaf("Delivered")),
                ("female".into(), leaf("Delivered feminine")),
                ("neuter".into(), leaf("Fallback")),
                ("alias".into(), leaf("Delivered")),
                ("other".into(), leaf("Explicit other")),
                ("_".into(), leaf("Fallback")),
            ],
        );
        assert_eq!(
            planned,
            Decision::Branch {
                selector: "selector",
                arms: vec![
                    DecisionArm {
                        keys: vec!["male".into(), "alias".into()],
                        result: leaf("Delivered"),
                    },
                    DecisionArm {
                        keys: vec!["female".into()],
                        result: leaf("Delivered feminine"),
                    },
                    DecisionArm {
                        keys: vec!["other".into()],
                        result: leaf("Explicit other"),
                    },
                ],
                fallback: Some(Box::new(leaf("Fallback"))),
            }
        );
        assert_eq!(
            Decision::branch(
                "mayThrow()",
                vec![("one".into(), leaf("Same")), ("other".into(), leaf("Same"))],
            ),
            Decision::Branch {
                selector: "mayThrow()",
                arms: vec![],
                fallback: Some(Box::new(leaf("Same"))),
            }
        );
        assert!(matches!(
            Decision::branch("key", vec![("one".into(), leaf("One"))]),
            Decision::Branch { fallback: None, .. }
        ));
    }
}
