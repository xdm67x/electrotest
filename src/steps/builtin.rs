use crate::steps::{
    registry::{ResolvedStep, StepDefinition},
    selector::WindowTarget,
};

pub fn definitions() -> Vec<StepDefinition> {
    vec![
        StepDefinition::new("click", click_match, ResolvedStep::click),
        StepDefinition::new(
            "switch_window_by_title",
            switch_window_by_title_match,
            |title| ResolvedStep::switch_window(WindowTarget::Title(title)),
        ),
        StepDefinition::new(
            "switch_window_by_index",
            switch_window_by_index_match,
            |index| ResolvedStep::switch_window(WindowTarget::Index(index.parse().unwrap_or(0))),
        ),
    ]
}

fn click_match(step_text: &str) -> Option<String> {
    let prefix = "When I click on \"";
    let suffix = '"';

    step_text
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
        .map(str::to_owned)
}

fn switch_window_by_title_match(step_text: &str) -> Option<String> {
    let prefix = "When I switch to window titled \"";
    let suffix = '"';

    step_text
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
        .map(str::to_owned)
}

fn switch_window_by_index_match(step_text: &str) -> Option<String> {
    let prefix = "When I switch to window index ";
    step_text.strip_prefix(prefix).map(str::to_owned)
}
