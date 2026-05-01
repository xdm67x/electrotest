---
name: electrotest-pilot
description: >
  Guides an AI agent to pilot an Electron application using the electrotest CLI.
  Translates natural language user requests into Gherkin .feature files,
  discovers the target Electron process, executes tests via CDP, and interprets results.
---

# Electrotest Pilot Skill

This skill enables an AI agent to automate an Electron application by generating and executing Gherkin `.feature` files using the `electrotest` CLI.

**Electrotest** is a Rust CLI tool that connects to a running Electron process via the Chrome DevTools Protocol (CDP) to execute navigation, interaction, and assertion steps written in Gherkin syntax.

## Required Tools

- `bash` — to run shell commands, discover processes, and execute the CLI.
- `read` — to inspect existing feature files or output directories.
- `write` — to create new `.feature` files.
- `edit` — to modify existing `.feature` files.

## Inputs

| Input | Required | Description |
|-------|----------|-------------|
| `user_request` | Yes | A natural language description of the test to perform. Example: *"Open the app, click the login button, wait 2 seconds, take a screenshot, and verify the dashboard heading is visible."* |
| `feature_path` | No | Path where the `.feature` file should be written. Defaults to `./test.feature` or a sensible location in the project. |
| `output_dir` | No | Directory for screenshots. Defaults to `./output`. |
| `pid` | No | The PID of the Electron process. If omitted, the agent must auto-discover it. |

## Outputs

1. A valid Gherkin `.feature` file written to disk.
2. Terminal output from the `electrotest` CLI execution.
3. A concise summary of passed/failed scenarios and any generated screenshots.

## Architecture Context

- **Entry point**: `electrotest --pid <PID> --features <PATH> --output-dir <DIR>`
- **Connection**: Discovers the Electron process, extracts `--remote-debugging-port`, connects via CDP over WebSocket.
- **Execution**: Runs each scenario step-by-step. `And`/`But` keywords resolve to the previous non-And/But keyword.
- **Screenshots**: Saved to the `--output-dir` as PNG files.

## Phase 1: Prerequisites

Before generating or running any feature file, verify:

1. **Electrotest binary is available**:
   ```bash
   which electrotest
   # or, if running from source in the electrotest repo:
   cargo build --release
   ./target/release/electrotest --version
   ```

2. **The Electron application is running with remote debugging enabled**:
   The target Electron process **must** have been started with `--remote-debugging-port=<PORT>` (commonly `9222` or `9223`).

   Verify with:
   ```bash
   ps aux | grep electron
   # Look for: --remote-debugging-port=9222
   ```

   If the flag is missing, instruct the user to restart the app:
   ```bash
   electron --remote-debugging-port=9222 .
   ```

## Phase 2: Natural Language → Gherkin Translation

Decompose the user's natural language request into discrete Gherkin steps.

### Step Mapping Dictionary

| User says | Gherkin Step | Keyword |
|-----------|-------------|---------|
| "open", "go to", "navigate to" a URL | `I navigate to "<URL>"` | `When` |
| "set window size", "resolution is" | `the window size is <W>x<H>` | `Given` |
| "click", "tap on" a button/link/element | `I click on "<text-or-selector>"` | `When` |
| "wait", "pause", "sleep for" N seconds | `I wait <N> seconds` | `And` |
| "take a screenshot", "capture" | `I take a screenshot "<filename>.png"` | `And` |
| "page should contain", "verify text" | `the page should contain "<text>"` | `Then` |
| "element ... should be visible" | `the element "<selector>" should be visible` | `Then` |
| "page title should be" | `the page title should be "<title>"` | `Then` |

### Keyword Chaining Rules

1. **Preconditions** (setup) start with `Given`.
2. **Actions** start with `When`.
3. **Assertions** start with `Then`.
4. **Subsequent steps** of the same type use `And`.
5. **Never** start a scenario with `And` or `Then`.
6. If the user mixes actions and assertions, group actions first (`Given`/`When`/`And`), then assertions (`Then`/`And`).

### Translation Example

**User Request:**
> "Open https://example.com in the app, set the window to 1280x720, click the 'Get Started' button, wait 1 second, take a screenshot called 'started.png', and verify the page contains 'Welcome'"

**Generated Feature File:**
```gherkin
Feature: Example App Smoke Test

  Scenario: Verify Get Started flow
    Given the window size is 1280x720

    When I navigate to "https://example.com"
    And I click on "Get Started"
    And I wait 1 seconds
    And I take a screenshot "started.png"

    Then the page should contain "Welcome"
```

## Phase 3: Electron PID Discovery

If the user did not provide a PID, auto-discover it using the following fallback chain:

**Method 1 — pgrep (preferred):**
```bash
pgrep -f "electron.*remote-debugging-port"
```

**Method 2 — lsof by port:**
```bash
lsof -iTCP -sTCP:LISTEN -P | grep -E "electron|:9222|:9223"
```

**Method 3 — ps aux parsing:**
```bash
ps aux | grep "[e]lectron.*remote-debugging-port"
```

If multiple PIDs are found, prefer the one whose command line includes `--remote-debugging-port`. If ambiguous, present the options to the user and ask them to specify.

## Phase 4: Write the Feature File

Use the `write` tool to create the `.feature` file.

**Template:**
```gherkin
Feature: <Brief description based on user request>

  Scenario: <Concise scenario name>
    Given <precondition>

    When <action>
    And <action>

    Then <assertion>
    And <assertion>
```

**Rules:**
- One blank line separates `Given`, `When`, and `Then` blocks for readability.
- Screenshot filenames should be descriptive and end in `.png`.
- Window size should be set early if mentioned by the user.

## Phase 5: Execute

Run `electrotest` with the discovered or provided PID:

```bash
# Binary mode
electrotest --pid <PID> --features <FEATURE_PATH> --output-dir <OUTPUT_DIR>

# Or from source (if inside the electrotest repo)
cargo run -- --pid <PID> --features <FEATURE_PATH> --output-dir <OUTPUT_DIR>
```

Alternatively, use the helper script:
```bash
./.agents/skills/electrotest-pilot/scripts/pilot.sh --features <FEATURE_PATH> --output-dir <OUTPUT_DIR>
```

## Phase 6: Interpret Results

Parse the CLI stdout and provide a concise summary:

- **Success:** Look for `✅ Scenario passed` and per-step `✓` markers.
- **Failure:** Look for `❌` followed by the scenario name and the specific step error.
- **Screenshots:** List all `.png` files found in the output directory.

**Example summary to return to the user:**
```
✅ Scenario passed: Verify Get Started flow
📸 Screenshots generated:
  - ./output/started.png
```

## Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| `No CDP targets available` | Electron not started with `--remote-debugging-port` | Restart the app with the flag |
| `Element '...' not found` | Selector/text mismatch or element not rendered | Add an `And I wait <N> seconds` step before the action |
| `WebSocket connection failed` | Wrong PID or port blocked | Re-run PID discovery, verify the process is alive |
| `No handler found for step` | Step text does not match any supported pattern | Check spelling and syntax against the mapping dictionary |
| Screenshot not saved | Output directory does not exist | `electrotest` creates it automatically; verify permissions |

## Important Notes

- **And/But resolution**: The runner automatically resolves `And` and `But` to the previous non-And/But keyword (`Given`, `When`, or `Then`). This means `And I click...` after a `When` is treated as a `When` step.
- **Click strategy**: The `click` handler first tries the input as a CSS selector (`document.querySelector`), then falls back to searching for an element whose text content exactly matches the input.
- **No integration tests**: `electrotest` requires a live Electron process. Do not attempt to run it in a headless CI context unless an Electron app is explicitly launched first.
