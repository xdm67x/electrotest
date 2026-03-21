# Electrotest V1 Design

## Goal

Build a Rust-first E2E testing tool for Electron applications, similar in spirit to Playwright for web apps, where users define test scenarios in Gherkin. The first version must support launching an Electron app, executing `.feature` scenarios against it, and collecting actionable failure artifacts.

## Product Scope

### In Scope for V1

- A single Rust CLI distributed as the main user-facing tool
- Project initialization and configuration
- Execution of Gherkin `.feature` files
- Automatic launch of the Electron application under test
- A Rust orchestration layer with a Node-based Playwright worker as the first execution engine
- Built-in Electron-oriented steps
- Custom project steps written in JavaScript or TypeScript
- Stable selector-first targeting with fallback strategies
- Basic multi-window support for Electron apps
- Failure artifacts such as screenshots, traces, and logs
- CI-friendly exit codes and terminal output

### Out of Scope for V1

- Visual scenario editor
- Record/replay authoring
- Advanced HTML reporting
- Rust plugin system for user-defined steps
- A non-Playwright execution backend
- Distributed execution or sophisticated parallel orchestration
- Complex native dialog and webview automation beyond the basic Playwright-backed path

## Runtime and Distribution Model

Electrotest V1 is distributed to users as a single Rust CLI binary. Users do not install a second CLI.

V1 still relies on Node.js as a runtime prerequisite because both the Playwright worker and custom `JS/TS` steps execute in a Node process. The supported model for V1 is:

- users install the `electrotest` Rust binary
- users must have a compatible system Node.js LTS available on `PATH`
- Electrotest owns the worker runtime assets and versioning
- on first use, Electrotest materializes a versioned internal worker directory in a tool-managed cache
- that internal worker contains the Playwright adapter and its pinned Node dependencies

This keeps the user-facing installation centered on a single Rust binary while avoiding a second user-managed CLI.

## User Experience

### Primary Commands

The CLI should expose a small, focused command set:

- `electrotest init` initializes project files and sample structure
- `electrotest test` executes selected scenarios
- `electrotest list` lists discovered features and scenarios
- `electrotest doctor` validates local setup and dependency readiness

The minimum `doctor` checks for V1 are:

- `Node.js` is installed and meets the supported version range
- the internal worker runtime is present or can be bootstrapped
- Playwright worker dependencies are available
- the configured Electron startup settings are valid
- configured `features/` and `steps/` paths exist

### Project Layout

The initial project setup should generate:

- `electrotest.toml` for project configuration
- `features/` for `.feature` files
- `steps/` for custom `JS/TS` step definitions

For TypeScript step authoring, `init` should also generate the minimum supporting files needed for the chosen execution path, such as a small `tsconfig.json` scoped to `steps/`.

### Scenario Authoring Model

Users write standard-looking Gherkin scenarios with an Electron-oriented step library. For example:

```gherkin
Feature: Settings
  Scenario: Open preferences
    Given the Electron app is launched
    When I click on "Settings"
    Then I should see "Preferences"
```

The V1 DSL should stay close to standard Gherkin while providing domain-friendly built-in steps for:

- clicking by text, role, or test id
- keyboard and mouse input
- visibility and content assertions
- window activation and multi-window navigation

## Architecture

### Recommended Approach

V1 uses a Rust CLI as the source of truth for product behavior, paired with a separate Node worker that executes Playwright Electron automation commands. This keeps the product Rust-first while using Playwright where it is already strong.

This approach was chosen over a pure Rust automation stack because it reduces delivery risk and improves initial reliability. It was chosen over a Node-first core because it preserves a clear long-term architecture in which Rust owns orchestration, execution policy, and product identity.

### Top-Level Components

#### CLI Layer

Responsible for:

- parsing commands and flags
- rendering terminal output
- controlling process exit codes
- exposing a stable user-facing interface

#### Configuration Layer

Responsible for reading and validating `electrotest.toml`, including:

- app startup mode and Electron launch settings
- feature discovery paths
- step definition paths
- timeout values
- artifact output settings

V1 supports two startup modes:

- `launch`: Electrotest starts the Electron app using configured command, args, cwd, and env
- `attach`: Electrotest connects only when the target app exposes the Playwright-compatible connection details required by the worker

`launch` is the default and primary path for V1. `attach` is supported only through explicit configuration and should stay intentionally narrow.

#### Gherkin Compiler

Responsible for:

- parsing `.feature` files
- building an internal executable scenario model
- resolving references to built-in and custom steps
- preparing scenario metadata for execution and reporting

#### Execution Runner

Responsible for:

- preparing the run environment
- starting or attaching to the Electron app
- coordinating scenario execution
- managing timeouts and result aggregation
- requesting failure artifacts
- producing the final run summary

#### Engine Abstraction

Rust must define an internal engine interface with operations such as:

- launch or attach to app
- list and target windows
- click elements
- fill inputs
- assert visibility or content
- capture screenshots and traces

This abstraction allows the first backend to be Playwright while preserving the option to add different execution engines later.

#### Playwright Engine Worker

The first engine implementation is a Node worker using Playwright's Electron support. The worker must remain narrowly focused on automation tasks. It should not become the place where scenario orchestration, product rules, or reporting logic live.

Its responsibilities are:

- launch the Electron app
- discover and switch windows
- execute low-level automation commands
- capture traces, screenshots, and raw runtime errors
- return normalized results to Rust

#### Step Extension Host

V1 supports custom steps written in JavaScript or TypeScript. These steps should run in a bounded execution environment that exposes engine primitives and scenario context, but not unrestricted access to the orchestration internals.

This gives projects flexibility without turning the extension mechanism into a second product architecture.

The V1 execution model for custom steps is:

- `.js` step files load directly in the Node runtime
- `.ts` step files are transpiled by the internal runtime to cached JavaScript before execution
- TypeScript transpilation is handled by the tool's internal runtime, not by requiring users to wire `ts-node` or `tsx`
- step modules register patterns and handlers through a small SDK owned by Electrotest

## Runtime Communication Model

Rust and the Node worker should communicate through a clear, structured process boundary, such as JSON messages over `stdin/stdout` or a similarly simple RPC-style transport.

For V1, this is preferred over `napi-rs` because it provides:

- simpler debugging
- stronger process isolation
- easier protocol testing
- cleaner future support for alternative backends

`napi-rs` is intentionally deferred unless a later version reveals a concrete need such as performance bottlenecks or a richer JavaScript SDK integration.

## Execution Flow

The expected flow for `electrotest test` is:

1. Load and validate project configuration
2. Discover `.feature` files
3. Parse and compile scenarios into an executable model
4. Start the Node Playwright worker
5. Launch or attach to the Electron app
6. Execute each step through either built-in Rust-backed behavior or custom `JS/TS` step definitions
7. Route engine actions through the Playwright worker
8. On failure, capture configured artifacts and normalize the error
9. Print a concise CLI summary and return a CI-friendly exit code

Rust remains the system of record for:

- Gherkin parsing
- step resolution
- orchestration decisions
- timeout policy
- error classification
- run summaries

Node remains an adapter runtime for the automation engine.

## Selector Strategy

V1 uses a hybrid targeting strategy with a strong preference for stable selectors. The resolution order should be:

1. explicit selector provided by the step
2. `data-testid`
3. accessible role and name
4. visible text

The built-in step library should encourage stable targeting rather than hiding fragile heuristics behind overly magical behavior.

## Window Model

The first version supports multiple windows within a single Electron application run. The execution context should track a current active window while allowing explicit selection or switching when scenarios need to target another window.

The minimum built-in behavior for V1 is:

- default to the first application window at launch
- allow switching by index
- allow switching by visible title or a stable window label when available
- report clearly when a requested window target is ambiguous or missing

This is enough to cover common Electron workflows without designing for the full complexity of native dialogs, embedded webviews, or cross-process orchestration in V1.

## Error Handling and Artifacts

Each failure should produce actionable output for both local debugging and CI use.

Minimum failure capture:

- screenshot of the active window
- failing step information
- normalized error message
- Playwright trace when enabled
- relevant logs when available

Errors should be classified into categories that help users act quickly:

- project configuration error
- missing step definition
- element not found
- timeout
- application or worker crash
- assertion failure

The CLI should show a concise human-readable summary, while detailed artifacts stay on disk.

## Testing Strategy for Electrotest Itself

### Rust Unit Tests

Cover:

- config parsing and validation
- Gherkin parsing and compilation
- step resolution
- error normalization and classification

### Integration Tests

Cover:

- Rust to Node worker protocol behavior
- launch of an Electron fixture app
- basic multi-window execution
- artifact generation on failure

### Fixture Projects

Maintain at least one small Electron fixture application to validate the real developer experience of the tool, not just isolated library behavior.

## Design Principles

- Keep Rust as the product core and source of truth
- Keep the Node worker minimal and replaceable
- Favor stable selectors over fuzzy matching
- Optimize for debuggability and CI usage before advanced reporting polish
- Provide extension points, but keep them bounded and intentional
- Avoid V1 features that expand scope without improving the core E2E workflow

## Open Decisions Deferred Beyond V1

These decisions are intentionally postponed and should not block planning for V1:

- whether to introduce `napi-rs` for selected cross-language integration points
- whether to add a visual scenario authoring experience
- whether to support alternative execution engines beyond Playwright
- whether to add richer reporting or scenario recording features

## Recommended Positioning

Electrotest should be positioned as a Rust-first E2E testing framework for Electron apps, with Gherkin scenario authoring and Playwright-powered execution under the hood.
