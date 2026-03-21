Feature: Multiple windows

  Scenario: Interact after opening a second window
    When I click on "#open-settings"
    When I switch to window titled "Preferences"
    When I switch to window index 1
    Given the fixture window title should be "Preferences"
