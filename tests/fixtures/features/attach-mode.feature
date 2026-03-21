Feature: Attach mode

  Scenario: Attach to an existing fixture app
    When I click on "#launch"
    Given the fixture window title should be "Fixture App"
