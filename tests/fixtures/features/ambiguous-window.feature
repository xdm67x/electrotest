Feature: Ambiguous window target

  Scenario: Report an ambiguous window target
    When I click on "#open-settings"
    When I click on "#open-settings"
    When I switch to window titled "Preferences"
