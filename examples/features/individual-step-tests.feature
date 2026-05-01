Feature: Individual Step Handler Tests

  Scenario: Test window size step
    Given the window size is 800x600

    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test navigate step
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test wait step
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 2 seconds

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test screenshot step
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds
    And I take a screenshot "test-screenshot.png"

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test click step by text
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds
    And I click on "👋"
    And I wait 1 seconds

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test click step by selector
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds
    And I click on "h1"
    And I wait 1 seconds

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test page contains assertion
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds

    Then the page should contain "Bonjour depuis le rendu d'Electron !"

  Scenario: Test element visible assertion
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds

    Then the element "h1" should be visible

  Scenario: Test page title assertion
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds

    Then the page title should be "Bonjour depuis le rendu d'Electron !"

  Scenario: Test type text step
    When I navigate to "file:///Users/memo/dev/electrotest/examples/electron-app/index.html"
    And I wait 1 seconds
    And I type "hello world" into "input"

    Then the page should contain "Bonjour depuis le rendu d'Electron !"
