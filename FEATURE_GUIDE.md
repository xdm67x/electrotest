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

### Assertions (Then)

- `Then the page should contain "<TEXT>"`
  - Verifies the page contains the text
  - Example: `Then the page should contain "Welcome"`

- `Then the element "<SELECTOR>" should be visible`
  - Verifies an element is visible
  - Example: `Then the element "#status" should be visible`

- `Then the page title should be "<TITLE>"`
  - Verifies the page title
  - Example: `Then the page title should be "Dashboard"`

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
