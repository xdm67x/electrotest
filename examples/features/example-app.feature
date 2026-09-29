Feature: Example Electron app smoke test
  Scenario: Verify the app window loads
    Given the window size is 800x600
    And I wait 1 seconds
    And I take a screenshot "01-app.png"
    Then the page should contain "Bonjour depuis le rendu d'Electron"
    And the page title should be "Bonjour depuis le rendu d'Electron !"
