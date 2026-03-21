#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledFeature {
    pub scenarios: Vec<CompiledScenario>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledScenario {
    pub feature_name: String,
    pub scenario_name: String,
    pub steps: Vec<CompiledStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledStep {
    pub text: String,
}
