use crate::gherkin::model::{CompiledFeature, CompiledScenario, CompiledStep};

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error(transparent)]
    Parse(#[from] ::gherkin::ParseError),
}

pub fn compile_str(input: &str) -> Result<CompiledFeature, CompileError> {
    let parsed = ::gherkin::Feature::parse(input, ::gherkin::GherkinEnv::default())?;
    compile_feature(parsed)
}

pub fn compile_feature(feature: ::gherkin::Feature) -> Result<CompiledFeature, CompileError> {
    let feature_name = feature.name;
    let scenarios = feature
        .scenarios
        .into_iter()
        .map(|scenario| CompiledScenario {
            feature_name: feature_name.clone(),
            scenario_name: scenario.name,
            steps: scenario
                .steps
                .into_iter()
                .map(|step| CompiledStep {
                    text: format!("{} {}", step.keyword.trim(), step.value),
                })
                .collect(),
        })
        .collect();

    Ok(CompiledFeature { scenarios })
}
