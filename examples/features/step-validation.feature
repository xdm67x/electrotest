Feature: Step Validation Test Suite

  This feature file tests all available Gherkin step handlers against the example Electron app.
  The example app must be running with --remote-debugging-port enabled.

  Scenario: Test navigation and window setup steps
    Given the window size is 1280x800

    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds

    Then the page title should be "Bonjour depuis le rendu d'Electron !"
    And the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test interaction steps - click and screenshot
    Given the window size is 1280x800

    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds
    And I take a screenshot "01-homepage.png"
    And I click on "👋"
    And I wait 1 seconds
    And I take a screenshot "02-after-click.png"

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test element visibility assertion
    Given the window size is 1280x800

    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds

    Then the element "h1" should be visible
    And the element "p" should be visible

  Scenario: Test type text step
    Given the window size is 1280x800

    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds
    And I type "test input" into "input"

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test wait step with different durations
    Given the window size is 1280x800

    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 0.5 seconds
    And I wait 1 seconds
    And I take a screenshot "03-after-wait.png"

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test complete workflow - all step types combined
    Given the window size is 1920x1080

    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds
    And I take a screenshot "04-full-workflow-start.png"
    And I click on "👋"
    And I wait 1 seconds
    And I take a screenshot "05-full-workflow-after-click.png"

    Then the page should contain "Bonjour depuis le rendu d'Electron !"
    And the element "h1" should be visible
    And the page title should be "Bonjour depuis le rendu d'Electron !"
