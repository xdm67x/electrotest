use crate::cli::feature::{Feature, Keyword, Scenario, Step};
use anyhow::{Context as _, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Parse a Gherkin .feature file
pub fn parse_feature_file(path: &Path) -> Result<Feature> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read feature file: {}", path.display()))?;
    parse_feature(&content)
}

/// Parse Gherkin content from a string
fn parse_feature(content: &str) -> Result<Feature> {
    let feature_re = Regex::new(r"^Feature:\s*(.+)$").unwrap();
    let scenario_re = Regex::new(r"^Scenario:\s*(.+)$").unwrap();
    let step_re = Regex::new(r"^(Given|When|Then|And|But)\s+(.+)$").unwrap();

    let mut feature_name: Option<String> = None;
    let mut scenarios: Vec<Scenario> = Vec::new();
    let mut current_scenario: Option<Scenario> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse Feature
        if let Some(caps) = feature_re.captures(line) {
            feature_name = Some(caps[1].trim().to_string());
            continue;
        }

        // Parse Scenario
        if let Some(caps) = scenario_re.captures(line) {
            // Save previous scenario if exists
            if let Some(scenario) = current_scenario.take() {
                scenarios.push(scenario);
            }

            current_scenario = Some(Scenario {
                name: caps[1].trim().to_string(),
                steps: Vec::new(),
            });
            continue;
        }

        // Parse Step
        if let Some(caps) = step_re.captures(line) {
            let keyword_str = &caps[1];
            let text = caps[2].trim().to_string();

            if let Some(keyword) = Keyword::from_str(keyword_str) {
                if let Some(ref mut scenario) = current_scenario {
                    scenario.steps.push(Step { keyword, text });
                } else {
                    return Err(anyhow::anyhow!(
                        "Step found outside of a Scenario: {}",
                        line
                    ));
                }
            }
        }
    }

    // Don't forget the last scenario
    if let Some(scenario) = current_scenario {
        scenarios.push(scenario);
    }

    match feature_name {
        Some(name) => Ok(Feature { name, scenarios }),
        None => Err(anyhow::anyhow!("No Feature declaration found in file")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_feature() {
        let content = r#"
Feature: Test de l'application

  Scenario: Vérifier la page
    Given the window size is 1920x1080
    When I navigate to "https://example.com"
    Then the page should contain "Welcome"
"#;

        let feature = parse_feature(content).unwrap();
        assert_eq!(feature.name, "Test de l'application");
        assert_eq!(feature.scenarios.len(), 1);

        let scenario = &feature.scenarios[0];
        assert_eq!(scenario.name, "Vérifier la page");
        assert_eq!(scenario.steps.len(), 3);
    }
}
