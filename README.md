# Electrotest

Electrotest is a Rust-first end-to-end testing tool for Electron applications.
It lets you describe scenarios in Gherkin, launch or attach to an Electron app, and execute tests through a Playwright-based worker runtime.

## What It Does Today

- Rust CLI with `init`, `doctor`, `list`, and `test`
- Gherkin feature parsing and scenario listing
- Launch mode for starting an Electron app under test
- Attach mode through a CDP endpoint
- Built-in steps for clicking and window switching
- Custom JavaScript and TypeScript step loading
- Failure artifacts such as `failure.png` and `trace.zip`

## Current Scope

This repository already contains a working vertical slice, but it is still an early version.

Supported built-in step shapes currently include:

- `When I click on "..."`
- `When I switch to window titled "..."`
- `When I switch to window index N`

Custom `JS/TS` steps can provide additional behavior using the worker-side step SDK.

## Requirements

- Rust toolchain
- Node.js `18`, `20`, or `22`
- npm

Playwright is installed in the bundled worker runtime during bootstrap.

## Getting Started

Initialize a project skeleton:

```bash
cargo run -- init --path ./my-electrotest-project
```

This creates:

- `electrotest.toml`
- `features/`
- `steps/`
- `tsconfig.json`

Default generated config:

```toml
[app]
mode = "launch"
command = "npm"
args = ["run", "start"]

[paths]
features = ["features"]
steps = ["steps"]
artifacts = ".electrotest/artifacts"
```

## CLI Commands

Show help:

```bash
cargo run -- --help
```

### `init`

Create project files:

```bash
cargo run -- init --path .
```

### `doctor`

Validate the local environment and project config:

```bash
cargo run -- doctor
```

`doctor` currently checks:

- Node.js availability
- supported Node.js version
- worker runtime presence
- Playwright worker dependency presence
- config startup validity
- configured feature and step paths

### `list`

List scenarios discovered from `.feature` files:

```bash
cargo run -- list --features features
```

Example output:

```text
Basic launch: Launch the fixture app
Attach mode: Attach to an existing fixture app
```

### `test`

Run scenarios from the current `electrotest.toml`:

```bash
cargo run -- test
```

If no subcommand is provided, the CLI defaults to `test`.

## Writing Features

Example feature:

```gherkin
Feature: Multiple windows

  Scenario: Interact after opening a second window
    When I click on "#open-settings"
    When I switch to window titled "Preferences"
    When I switch to window index 1
```

## Writing Custom Steps

Put custom steps in `steps/`.

Example TypeScript step registration:

```ts
export type StepContext = {
  appTitle(): Promise<string>;
};

export function registerSampleSteps(
  register: (pattern: string, handler: (ctx: StepContext, expected: string) => Promise<void>) => void,
) {
  register("the fixture window title should be {string}", async (ctx, expected) => {
    const actual = await ctx.appTitle();

    if (actual !== expected) {
      throw new Error(`expected fixture window title to be ${expected}, got ${actual}`);
    }
  });
}
```

Custom step patterns are matched alongside built-in steps during scenario execution.

## Launch vs Attach

### Launch mode

Electrotest starts the app itself:

```toml
[app]
mode = "launch"
command = "npm"
args = ["run", "start"]
```

### Attach mode

Electrotest connects to an already-running app through a CDP endpoint:

```toml
[app]
mode = "attach"
endpoint = "http://127.0.0.1:9222"
```

Or via endpoint file:

```toml
[app]
mode = "attach"
endpoint_file = ".electrotest/attach-endpoint.txt"
```

## Artifacts

On scenario failure, Electrotest writes artifacts into the configured artifacts directory.
The current runner writes at least:

- `failure.png`
- `trace.zip`

By default, artifacts go under:

```text
.electrotest/artifacts/
```

## Development

Run the full test suite:

```bash
cargo test
```

Useful targeted commands:

```bash
cargo test --test cli_init -- --nocapture
cargo test --test cli_doctor -- --nocapture
cargo test --test e2e_fixture -- --nocapture
```

## Releases

Electrotest uses a two-step release flow:

1. run the `create-release-pr` workflow with a version like `0.1.0`
2. review and merge the generated release PR
3. push tag `v0.1.0`

The tag-driven release workflow publishes raw binaries for:

- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-linux-musl`

Published assets are direct binaries plus `SHA256SUMS.txt`; the release process does not produce `zip` or `tar.gz` archives.

Before relying on the automation, maintainers should verify all GitHub-side release paths:

- a successful `create-release-pr` run with valid `X.Y.Z` input
- an invalid release PR input case that fails early
- a successful tag release where `vX.Y.Z` matches `Cargo.toml`
- a mismatch tag case that fails before publishing
- uploaded assets named exactly `electrotest-vX.Y.Z-aarch64-apple-darwin`, `electrotest-vX.Y.Z-x86_64-pc-windows-msvc.exe`, `electrotest-vX.Y.Z-x86_64-unknown-linux-musl`, and `SHA256SUMS.txt`

## Repository Layout

- `src/cli/` - CLI parsing and command entrypoints
- `src/config/` - project config parsing and validation
- `src/gherkin/` - feature parsing and compilation
- `src/steps/` - built-in step registry and selector normalization
- `src/engine/` - Rust-side worker protocol and Playwright adapter
- `src/runner/` - scenario execution, error handling, and artifacts
- `runtime/worker/` - Node/Playwright worker runtime
- `tests/fixtures/` - Electron app and fixture scenarios

## Limitations

Current implementation is intentionally narrow:

- built-in step coverage is still small
- reporting is terminal-focused
- the execution model is optimized around the current Playwright worker
- the feature set is driven by the existing tested fixture flows

## License

No license file is included yet.
