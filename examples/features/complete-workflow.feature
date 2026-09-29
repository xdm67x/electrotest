Feature: Complete workflow demonstration
  Scenario: Fill a form and validate the result
    Given the window size is 1280x800
    When I navigate to "file:///path/to/app/index.html"
    And I wait 1 seconds
    And I scroll down by 200 pixels
    And I type "Alice" into "#name-input"
    And I clear "#name-input"
    And I type "Bob" into "#name-input"
    And I select "fr" from "#language"
    And I press "Enter" on "#name-input"
    And I scroll to "#footer"
    And I scroll to top
    And I scroll to bottom
    And I hover over "#menu"
    And I double-click on "#item"
    And I click on "#submit-btn"
    And I wait for "#result"
    And I take a screenshot "01-result.png"
    Then the page should contain "Success"
    And the page should not contain "Error"
    And the element "#result" should be visible
    And the element "#result" should exist
    And the element "#result" should contain "Success"
    And the value of "#name-input" should be "Bob"
    And the element "#submit-btn" should be disabled
    And I should see 1 elements "#result"
    And the page title should be "Dashboard"
    And the URL should contain "success"
