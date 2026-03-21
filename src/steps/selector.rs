#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepTarget {
    pub label: String,
    pub explicit_selector: Option<String>,
}

impl StepTarget {
    pub fn from_capture(label: String) -> Self {
        let explicit_selector = looks_like_selector(&label).then(|| label.clone());
        Self {
            label,
            explicit_selector,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowTarget {
    Title(String),
    Index(usize),
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

fn looks_like_selector(value: &str) -> bool {
    value.starts_with('#') || value.starts_with('.') || value.starts_with('[')
}
