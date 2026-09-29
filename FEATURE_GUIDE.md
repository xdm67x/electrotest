# Gherkin Feature Guide

## `.feature` File Syntax

Feature files use standard Gherkin syntax with Given/When/Then keywords.

## Available Steps

### Setup (Given)

- `Given the window size is <WIDTH>x<HEIGHT>`
  - Sets the Electron window size
  - Example: `Given the window size is 1920x1080`

### Actions (When)

- `When I navigate to "<URL>"`
  - Navigates to a URL
  - Example: `When I navigate to "https://example.com"`

- `When I click on button "<TEXT>"`
  - Clicks a button containing the specified text
  - Example: `When I click on button "Submit"`

- `When I click on "<SELECTOR>"`
  - Clicks an element via CSS selector
  - Example: `When I click on "#submit-btn"`

- `When I take a screenshot "<FILENAME>"`
  - Takes a screenshot
  - Example: `When I take a screenshot "01-homepage.png"`

- `When I wait <N> seconds`
  - Waits for N seconds
  - Example: `When I wait 2 seconds`
- `When I wait for "<SELECTOR>"`
  - Waits for an element to appear in the DOM (10s timeout)
  - Example: `When I wait for "#result"`
- `When I type "<TEXT>" into "<SELECTOR>"`
  - Types text into an input, dispatching keydown/keypress/input/change events
  - Example: `When I type "hello" into "#name-input"`
- `When I clear "<SELECTOR>"`
  - Clears the value of an input or textarea
  - Example: `When I clear "#name-input"`
- `When I press "<KEY>"`
  - Presses a key on the focused element (or document body)
  - Example: `When I press "Enter"`
- `When I press "<KEY>" on "<SELECTOR>"`
  - Focuses the element and presses a key on it
  - Example: `When I press "Enter" on "#search-input"`
- `When I double-click on "<SELECTOR>"`
  - Double-clicks an element via CSS selector
  - Example: `When I double-click on "#item"`
- `When I hover over "<SELECTOR>"`
  - Hovers over an element (mouseover/mouseenter/mousemove events)
  - Example: `When I hover over "#menu"`
- `When I select "<VALUE>" from "<SELECTOR>"`
  - Selects an option from a `<select>` dropdown by value or label
  - Example: `When I select "fr" from "#language"`
- `When I scroll down by <N> pixels` / `When I scroll up by <N> pixels`
  - Scrolls the page by N pixels
  - Example: `When I scroll down by 500 pixels`
- `When I scroll to "<SELECTOR>"`
  - Scrolls an element into view
  - Example: `When I scroll to "#footer"`
- `When I scroll to top` / `When I scroll to bottom`
  - Scrolls to the top or bottom of the page
  - Example: `When I scroll to bottom`
- `When I reload the page`
  - Reloads the current page
  - Example: `When I reload the page`

### Assertions (Then)

- `Then the page should contain "<TEXT>"`
  - Verifies the page contains the text
  - Example: `Then the page should contain "Welcome"`

- `Then the element "<SELECTOR>" should be visible`
  - Verifies an element is visible
  - Example: `Then the element "#status" should be visible`

- `Then the page should not contain "<TEXT>"`
  - Verifies the page does not contain the text
  - Example: `Then the page should not contain "Error"`
- `Then the page title should be "<TITLE>"`
  - Verifies the page title
  - Example: `Then the page title should be "Dashboard"`
- `Then the element "<SELECTOR>" should not be visible`
  - Verifies an element is not visible
  - Example: `Then the element "#modal" should not be visible`
- `Then the element "<SELECTOR>" should exist` / `should not exist`
  - Verifies an element is present in (or absent from) the DOM
  - Example: `Then the element "#logo" should exist`
- `Then the element "<SELECTOR>" should contain "<TEXT>"`
  - Verifies an element's text content
  - Example: `Then the element "#status" should contain "Active"`
- `Then the value of "<SELECTOR>" should be "<VALUE>"`
  - Verifies the value of an input, textarea, or select
  - Example: `Then the value of "#name-input" should be "Alice"`
- `Then the element "<SELECTOR>" should be enabled` / `should be disabled`
  - Verifies an element's enabled/disabled state
  - Example: `Then the element "#submit-btn" should be enabled`
- `Then I should see <N> elements "<SELECTOR>"`
  - Verifies the number of matching elements
  - Example: `Then I should see 3 elements ".item"`
- `Then the URL should be "<URL>"` / `Then the URL should contain "<TEXT>"`
  - Verifies the current page URL
  - Example: `Then the URL should contain "dashboard"`

### And/But Keywords

The `And` and `But` keywords can follow any step type and inherit the previous type.

## Complete Example

```gherkin
Feature: Electron Application Test

  Scenario: Check the homepage
    Given the window size is 1920x1080

    When I navigate to "https://example.com"
    And I take a screenshot "01-homepage.png"
    And I click on button "Submit"
    And I wait 1 seconds
    And I take a screenshot "02-after-submit.png"

    Then the page should contain "Welcome"
    And the element "#status" should be visible
    And the page title should be "Dashboard"
```

## Execution

```bash
# Connect to an Electron process with CDP
./electrotest --pid 12345 --features tests/my-test.feature

# With custom output directory
./electrotest --pid 12345 --features tests/test.feature --output-dir ./screenshots
```

**Note:** The Electron process must have been started with `--remote-debugging-port=XXXX`.

## Extensibility

To add a new step, create a struct implementing `StepHandler` in `src/cli/steps/`:

```rust
use async_trait::async_trait;
use crate::cli::steps::StepHandler;

pub struct MyNewStep;

#[async_trait]
impl StepHandler for MyNewStep {
    fn can_handle(&self, step: &Step) -> bool {
        step.keyword.is_when_type() && step.text.contains("my pattern")
    }

    async fn execute(&self, step: &Step, ctx: &mut Context) -> Result<()> {
        // Step logic
        ctx.cdp_client.evaluate("...").await?;
        Ok(())
    }
}
```

Then register it in `StepRegistry::new()` in `src/cli/steps/mod.rs`.
