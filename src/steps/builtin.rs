use crate::steps::registry::{ResolvedStep, StepDefinition};

pub fn definitions() -> Vec<StepDefinition> {
    vec![StepDefinition::new(
        "click",
        click_match,
        ResolvedStep::click,
    )]
}

fn click_match(step_text: &str) -> Option<String> {
    let prefix = "When I click on \"";
    let suffix = '"';

    step_text
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
        .map(str::to_owned)
}
