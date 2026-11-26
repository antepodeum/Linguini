//! Backend-independent lowering of form and function dispatch.

use linguini_ir::{IrBranch, IrFunctionBranch, IrFunctionBranchValue, IrFunctionParameter, IrText};

use crate::Decision;

/// Project form results before merging equivalent branches. Empty key lists are defaults.
pub fn form_dispatch_plan<Selector: Clone + PartialEq, Result: Clone + PartialEq>(
    branches: &[IrBranch],
    selector: Selector,
    mut project_text: impl FnMut(&IrText) -> Result,
) -> Decision<Selector, Result> {
    Decision::branch(
        selector,
        branches
            .iter()
            .flat_map(|branch| {
                let value = Decision::Return(project_text(&branch.value));
                let keys = if branch.keys.is_empty() {
                    vec!["_".to_owned()]
                } else {
                    branch.keys.clone()
                };
                keys.into_iter().map(move |key| (key, value.clone()))
            })
            .collect(),
    )
}

/// Lower unnamed dispatch dimensions using their full parameter indices.
/// Named payload parameters do not consume a dimension. A missing selector is projected
/// as `None`, unless a statically selected `undefined`, `_`, or `other` body resolves it.
/// Text is projected before equivalent results are merged, keeping backend equality semantics.
pub fn function_dispatch_plan<Selector: Clone + PartialEq, Result: Clone + PartialEq>(
    parameters: &[IrFunctionParameter],
    branches: &[IrFunctionBranch],
    mut project_selector: impl FnMut(Option<usize>) -> Selector,
    mut project_text: impl FnMut(&IrText) -> Result,
) -> Decision<Selector, Result> {
    let selectors = parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| parameter.name.is_none().then_some(index))
        .collect::<Vec<_>>();
    lower_function_dispatch(
        &selectors,
        branches,
        0,
        &mut project_selector,
        &mut project_text,
    )
}

fn lower_function_dispatch<Selector: Clone + PartialEq, Result: Clone + PartialEq>(
    selectors: &[usize],
    branches: &[IrFunctionBranch],
    depth: usize,
    project_selector: &mut impl FnMut(Option<usize>) -> Selector,
    project_text: &mut impl FnMut(&IrText) -> Result,
) -> Decision<Selector, Result> {
    let selector = selectors.get(depth).copied();
    if selector.is_none() {
        if let Some(branch) = branches
            .iter()
            .find(|branch| branch.key == "undefined")
            .or_else(|| branches.iter().find(|branch| branch.key == "_"))
            .or_else(|| branches.iter().find(|branch| branch.key == "other"))
        {
            return lower_function_value(
                selectors,
                &branch.value,
                depth,
                project_selector,
                project_text,
            );
        }
    }
    let selector = project_selector(selector);
    Decision::branch(
        selector,
        branches
            .iter()
            .map(|branch| {
                let result = lower_function_value(
                    selectors,
                    &branch.value,
                    depth,
                    project_selector,
                    project_text,
                );
                (branch.key.clone(), result)
            })
            .collect(),
    )
}

fn lower_function_value<Selector: Clone + PartialEq, Result: Clone + PartialEq>(
    selectors: &[usize],
    value: &IrFunctionBranchValue,
    depth: usize,
    project_selector: &mut impl FnMut(Option<usize>) -> Selector,
    project_text: &mut impl FnMut(&IrText) -> Result,
) -> Decision<Selector, Result> {
    match value {
        IrFunctionBranchValue::Text(text) => Decision::Return(project_text(text)),
        IrFunctionBranchValue::Dispatch(children) => lower_function_dispatch(
            selectors,
            children,
            depth + 1,
            project_selector,
            project_text,
        ),
    }
}

#[cfg(test)]
mod tests;
