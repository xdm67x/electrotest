use crate::steps::{builtin, selector::StepTarget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStep {
    action_name: &'static str,
    target: Option<StepTarget>,
}

impl ResolvedStep {
    pub fn click(label: String) -> Self {
        Self {
            action_name: "click",
            target: Some(StepTarget {
                label,
                explicit_selector: None,
            }),
        }
    }

    pub fn action_name(&self) -> &str {
        self.action_name
    }

    pub fn target(&self) -> Option<&StepTarget> {
        self.target.as_ref()
    }
}

pub struct Registry {
    definitions: Vec<StepDefinition>,
}

impl Registry {
    pub fn builtin() -> Self {
        Self {
            definitions: builtin::definitions(),
        }
    }

    pub fn resolve(&self, step_text: &str) -> Option<ResolvedStep> {
        self.definitions.iter().find_map(|definition| {
            (definition.matcher)(step_text).map(|capture| (definition.builder)(capture))
        })
    }
}

pub(crate) struct StepDefinition {
    matcher: fn(&str) -> Option<String>,
    builder: fn(String) -> ResolvedStep,
}

impl StepDefinition {
    pub(crate) fn new(
        _action_name: &'static str,
        matcher: fn(&str) -> Option<String>,
        builder: fn(String) -> ResolvedStep,
    ) -> Self {
        Self { matcher, builder }
    }
}
