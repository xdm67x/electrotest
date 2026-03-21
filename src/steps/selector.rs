#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepTarget {
    pub label: String,
    pub explicit_selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Locator {
    Explicit(String),
    TestId(String),
    RoleName { role: String, name: String },
    Text(String),
}

pub fn normalize_target(raw: StepTarget) -> Vec<Locator> {
    let StepTarget {
        label,
        explicit_selector,
    } = raw;

    let mut locators = Vec::new();

    if let Some(selector) = explicit_selector {
        locators.push(Locator::Explicit(selector));
    }

    locators.push(Locator::TestId(label.clone()));
    locators.push(Locator::RoleName {
        role: "button".into(),
        name: label.clone(),
    });
    locators.push(Locator::Text(label));

    locators
}
