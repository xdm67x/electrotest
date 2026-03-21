#[test]
fn compiles_feature_into_executable_scenario() {
    let feature = r#"
        Feature: Settings
          Scenario: Open preferences
            Given the Electron app is launched
            When I click on "Settings"
            Then I should see "Preferences"
    "#;

    let compiled = electrotest::gherkin::compile_str(feature).unwrap();
    assert_eq!(compiled.scenarios.len(), 1);
    assert_eq!(compiled.scenarios[0].feature_name, "Settings");
    assert_eq!(compiled.scenarios[0].scenario_name, "Open preferences");
    assert_eq!(compiled.scenarios[0].steps.len(), 3);
    assert_eq!(
        compiled.scenarios[0].steps[0].text,
        "Given the Electron app is launched"
    );
    assert_eq!(
        compiled.scenarios[0].steps[1].text,
        "When I click on \"Settings\""
    );
    assert_eq!(
        compiled.scenarios[0].steps[2].text,
        "Then I should see \"Preferences\""
    );
}

#[test]
fn returns_parse_error_for_invalid_feature() {
    let error = electrotest::gherkin::compile_str("Scenario: Missing feature").unwrap_err();
    assert!(matches!(
        error,
        electrotest::gherkin::CompileError::Parse(_)
    ));
}

#[test]
fn resolves_click_step_to_builtin_handler() {
    let registry = electrotest::steps::Registry::builtin();
    let step = registry.resolve("When I click on \"Settings\"").unwrap();
    assert_eq!(step.action_name(), "click");
}

#[test]
fn normalizes_step_target_into_v1_locator_order() {
    let target = electrotest::steps::StepTarget {
        label: "Settings".into(),
        explicit_selector: Some("#settings".into()),
    };

    let locators = electrotest::steps::normalize_target(target);

    assert_eq!(
        locators,
        vec![
            electrotest::steps::Locator::Explicit("#settings".into()),
            electrotest::steps::Locator::TestId("Settings".into()),
            electrotest::steps::Locator::RoleName {
                role: "button".into(),
                name: "Settings".into(),
            },
            electrotest::steps::Locator::Text("Settings".into()),
        ]
    );
}
