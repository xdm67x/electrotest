use std::path::{Path, PathBuf};

pub async fn run(args: crate::cli::ListArgs) -> Result<(), crate::Error> {
    for scenario in load_scenarios(&args.features)? {
        println!("{}: {}", scenario.feature_name, scenario.scenario_name);
    }

    Ok(())
}

fn load_scenarios(feature_paths: &[PathBuf]) -> Result<Vec<crate::gherkin::CompiledScenario>, crate::Error> {
    let mut scenarios = Vec::new();

    for feature_path in feature_paths {
        if feature_path.is_dir() {
            for entry in std::fs::read_dir(feature_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("feature") {
                    scenarios.extend(load_scenarios_from_file(&path)?);
                }
            }
        } else {
            scenarios.extend(load_scenarios_from_file(feature_path)?);
        }
    }

    Ok(scenarios)
}

fn load_scenarios_from_file(path: &Path) -> Result<Vec<crate::gherkin::CompiledScenario>, crate::Error> {
    let raw = std::fs::read_to_string(path)?;
    Ok(crate::gherkin::compile_str(&raw)?.scenarios)
}
