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

    pub fn custom() -> Self {
        Self {
            action_name: "custom",
            target: None,
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

    pub fn with_custom_patterns(patterns: Vec<String>) -> Self {
        let mut definitions = builtin::definitions();
        definitions.extend(patterns.into_iter().map(StepDefinition::custom));
        Self { definitions }
    }

    pub fn resolve(&self, step_text: &str) -> Option<ResolvedStep> {
        self.definitions.iter().find_map(|definition| {
            (definition.matcher)(step_text).map(|capture| (definition.builder)(capture))
        })
    }
}

pub(crate) struct StepDefinition {
    matcher: Box<dyn Fn(&str) -> Option<String> + Send + Sync>,
    builder: Box<dyn Fn(String) -> ResolvedStep + Send + Sync>,
}

impl StepDefinition {
    pub(crate) fn new(
        _action_name: &'static str,
        matcher: fn(&str) -> Option<String>,
        builder: fn(String) -> ResolvedStep,
    ) -> Self {
        Self {
            matcher: Box::new(matcher),
            builder: Box::new(builder),
        }
    }

    fn custom(pattern: String) -> Self {
        Self {
            matcher: Box::new(move |step_text| match_expression_pattern(&pattern, step_text)),
            builder: Box::new(|_| ResolvedStep::custom()),
        }
    }
}

fn match_expression_pattern(pattern: &str, step_text: &str) -> Option<String> {
    let candidate = step_text
        .strip_prefix("Given ")
        .or_else(|| step_text.strip_prefix("When "))
        .or_else(|| step_text.strip_prefix("Then "))
        .or_else(|| step_text.strip_prefix("And "))
        .or_else(|| step_text.strip_prefix("But "))
        .unwrap_or(step_text);

    let mut remaining_pattern = pattern;
    let mut remaining_step = candidate;

    while let Some(index) = remaining_pattern.find("{string}") {
        let prefix = &remaining_pattern[..index];
        if !remaining_step.starts_with(prefix) {
            return None;
        }

        remaining_step = &remaining_step[prefix.len()..];
        if !remaining_step.starts_with('"') {
            return None;
        }

        remaining_step = &remaining_step[1..];
        let closing_quote = remaining_step.find('"')?;
        remaining_step = &remaining_step[closing_quote + 1..];
        remaining_pattern = &remaining_pattern[index + "{string}".len()..];
    }

    if remaining_step == remaining_pattern {
        Some(candidate.to_owned())
    } else {
        None
    }
}
