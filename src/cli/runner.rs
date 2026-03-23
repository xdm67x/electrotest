use crate::cli::context::Context;
use crate::cli::feature::{Feature, Keyword, Scenario, Step};
use crate::cli::steps::StepRegistry;
use anyhow::Result;

/// Runs Gherkin features against an Electron application
pub struct FeatureRunner {
    registry: StepRegistry,
}

/// Result of running a scenario
pub struct ScenarioResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

/// Result of running a feature
pub struct FeatureResult {
    #[allow(dead_code)]
    pub name: String,
    pub scenarios: Vec<ScenarioResult>,
}

impl FeatureRunner {
    /// Create a new runner with the default step registry
    pub fn new() -> Self {
        Self {
            registry: StepRegistry::new(),
        }
    }

    /// Run a feature file
    pub async fn run_feature(
        &self,
        feature: &Feature,
        ctx: &mut Context,
    ) -> Result<FeatureResult> {
        println!("\n📋 Feature: {}", feature.name);
        println!("{}", "=".repeat(50));

        let mut scenario_results = Vec::new();

        for scenario in &feature.scenarios {
            let result = self.run_scenario(scenario, ctx).await;
            scenario_results.push(result);
        }

        // Print summary
        self.print_summary(&scenario_results);

        Ok(FeatureResult {
            name: feature.name.clone(),
            scenarios: scenario_results,
        })
    }

    /// Run a single scenario
    async fn run_scenario(
        &self,
        scenario: &Scenario,
        ctx: &mut Context,
    ) -> ScenarioResult {
        println!("\n  📝 Scenario: {}", scenario.name);

        let mut last_keyword = Keyword::Given;

        for step in &scenario.steps {
            // Resolve "And" and "But" keywords to their parent type
            let effective_keyword = match step.keyword {
                Keyword::And | Keyword::But => last_keyword,
                other => other,
            };

            // Find handler
            let step_with_keyword = Step {
                keyword: effective_keyword,
                text: step.text.clone(),
            };

            match self.registry.find_handler(&step_with_keyword) {
                Some(handler) => {
                    match handler.execute(&step_with_keyword, ctx).await {
                        Ok(_) => {
                            // Step passed
                        }
                        Err(e) => {
                            return ScenarioResult {
                                name: scenario.name.clone(),
                                passed: false,
                                error: Some(format!(
                                    "{} {} - {}",
                                    step.keyword, step.text, e
                                )),
                            };
                        }
                    }
                }
                None => {
                    return ScenarioResult {
                        name: scenario.name.clone(),
                        passed: false,
                        error: Some(format!(
                            "No handler found for step: {} {}",
                            step.keyword, step.text
                        )),
                    };
                }
            }

            // Track the last non-And/But keyword
            if !matches!(step.keyword, Keyword::And | Keyword::But) {
                last_keyword = step.keyword;
            }
        }

        println!("  ✅ Scenario passed");
        ScenarioResult {
            name: scenario.name.clone(),
            passed: true,
            error: None,
        }
    }

    /// Print a summary of the test run
    fn print_summary(
        &self,
        results: &[ScenarioResult],
    ) {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();

        println!("\n{}", "=".repeat(50));
        println!("📊 Summary: {} passed, {} failed", passed, failed);

        for result in results {
            if !result.passed {
                if let Some(ref error) = result.error {
                    println!("  ❌ {}: {}", result.name, error);
                }
            }
        }
    }
}

impl Default for FeatureRunner {
    fn default() -> Self {
        Self::new()
    }
}
