pub async fn run(args: crate::cli::ListArgs) -> Result<(), crate::Error> {
    for scenario in crate::gherkin::load_scenarios(&args.features)? {
        println!("{}: {}", scenario.feature_name, scenario.scenario_name);
    }

    Ok(())
}
