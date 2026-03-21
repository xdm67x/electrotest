# Electrotest V1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust-first CLI that can initialize an Electrotest project, validate local prerequisites, parse `.feature` files, launch an Electron app through a Node Playwright worker, execute built-in and custom `JS/TS` steps, and emit actionable failure artifacts.

**Architecture:** Rust owns the CLI, config, Gherkin compilation, step resolution, orchestration, error handling, and reporting. A versioned Node worker is bootstrapped by Rust into a tool-managed cache and acts as a narrow Playwright adapter over a JSON protocol. Custom `JS/TS` steps run inside that Node runtime through a bounded SDK.

**Tech Stack:** Rust 2024, `clap`, `tokio`, `serde`, `toml`, `thiserror`, `camino`, `assert_cmd`, `tempfile`, Gherkin parser crate, Node.js LTS, TypeScript, Playwright Electron.

---

## Proposed File Structure

### Rust application files

- `Cargo.toml` - declare runtime and test dependencies
- `src/main.rs` - CLI entrypoint that delegates to the library
- `src/lib.rs` - top-level module exports and shared `Result` type
- `src/cli/mod.rs` - command parsing and dispatch
- `src/cli/commands/init.rs` - `electrotest init` implementation
- `src/cli/commands/doctor.rs` - `electrotest doctor` implementation
- `src/cli/commands/list.rs` - `electrotest list` implementation
- `src/cli/commands/test.rs` - `electrotest test` implementation
- `src/config/mod.rs` - config loading API
- `src/config/types.rs` - serde-backed config structs
- `src/config/validate.rs` - validation rules for launch, attach, paths, and timeouts
- `src/project/templates.rs` - embedded default templates for `init`
- `src/project/bootstrap.rs` - worker cache/bootstrap logic
- `src/gherkin/mod.rs` - parser/compiler public API
- `src/gherkin/model.rs` - executable scenario model
- `src/gherkin/compile.rs` - transform parsed Gherkin into executable scenarios
- `src/steps/mod.rs` - step system entrypoints
- `src/steps/builtin.rs` - built-in Electron-oriented step catalog
- `src/steps/registry.rs` - merge built-in and custom step definitions
- `src/steps/selector.rs` - selector strategy and normalized locator model
- `src/engine/mod.rs` - engine trait and shared types
- `src/engine/protocol.rs` - JSON message schema for Rust <-> Node worker
- `src/engine/process.rs` - spawn and communicate with the worker process
- `src/engine/playwright.rs` - Playwright-backed engine adapter
- `src/runner/mod.rs` - runner public API
- `src/runner/context.rs` - run context, active window state, and scenario state
- `src/runner/artifacts.rs` - screenshot, trace, and log persistence helpers
- `src/runner/errors.rs` - normalized error types and classification
- `src/runner/execute.rs` - scenario execution loop

### Runtime assets shipped with Electrotest

- `runtime/worker/package.json` - pinned Node dependencies for the internal worker
- `runtime/worker/tsconfig.json` - TypeScript build config for runtime sources
- `runtime/worker/src/index.ts` - worker process entrypoint
- `runtime/worker/src/protocol.ts` - worker-side protocol definitions
- `runtime/worker/src/engine.ts` - Playwright Electron command implementation
- `runtime/worker/src/steps.ts` - custom step loader and SDK bridge
- `runtime/worker/src/sdk.ts` - minimal SDK exposed to user `JS/TS` step modules
- `runtime/worker/scripts/build.mjs` - compile worker TypeScript into runnable JavaScript

### Tests and fixtures

- `tests/cli_init.rs` - project initialization behavior
- `tests/cli_doctor.rs` - doctor prerequisite checks
- `tests/config_loading.rs` - config parsing and validation coverage
- `tests/gherkin_compile.rs` - scenario compilation coverage
- `tests/worker_protocol.rs` - process protocol integration coverage
- `tests/runner_errors.rs` - artifact and error classification behavior
- `tests/e2e_fixture.rs` - full Electron fixture execution coverage
- `tests/support/mod.rs` - shared test helpers
- `tests/fixtures/electron-app/` - minimal Electron app fixture
- `tests/fixtures/features/` - sample feature files for integration tests
- `tests/fixtures/steps/` - sample custom `JS/TS` step definitions

## Implementation Notes

- Keep the Node worker bootstrapping simple: copy the versioned runtime into a cache directory, run `npm install` once during bootstrap, and reuse it across runs.
- Build the worker during bootstrap by running `node runtime/worker/scripts/build.mjs`, and execute the compiled output from `dist/index.js`.
- Use a Gherkin parser crate instead of hand-parsing `.feature` files.
- Keep built-in steps intentionally small in V1: app launch, click, fill, key press, visible text assertion, window switch.
- Support `attach` mode only in config and engine plumbing; do not add extra UX beyond what the spec requires.
- Default artifact root should be `.electrotest/artifacts/<timestamp>/`.
- Add `.superpowers/` and `.electrotest/` to `.gitignore` during `init` if not already present.

### Task 1: Establish the Rust CLI foundation

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/cli/mod.rs`
- Test: `tests/cli_init.rs`

- [ ] **Step 1: Write the failing CLI smoke test**

```rust
use assert_cmd::Command;

#[test]
fn prints_help_for_top_level_cli() {
    Command::cargo_bin("electrotest")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("electrotest"));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test cli_init prints_help_for_top_level_cli -- --exact`
Expected: FAIL because `assert_cmd`/`predicates` are missing or the binary does not expose the expected CLI.

- [ ] **Step 3: Add minimal CLI dependencies and command skeleton**

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "process", "fs"] }
thiserror = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

```rust
// src/main.rs
#[tokio::main]
async fn main() {
    if let Err(err) = electrotest::cli::run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
```

```rust
// src/cli/mod.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "electrotest")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Doctor,
    List,
    Test,
}

pub async fn run() -> Result<(), crate::Error> {
    let _ = Cli::parse();
    Ok(())
}
```

- [ ] **Step 4: Run the smoke test again**

Run: `cargo test --test cli_init prints_help_for_top_level_cli -- --exact`
Expected: PASS.

- [ ] **Step 5: Commit the CLI skeleton**

```bash
git add Cargo.toml src/main.rs src/lib.rs src/cli/mod.rs tests/cli_init.rs
git commit -m "feat: add electrotest cli skeleton"
```

### Task 2: Implement `electrotest init` and project templates

**Files:**
- Modify: `src/cli/mod.rs`
- Create: `src/cli/commands/init.rs`
- Create: `src/project/templates.rs`
- Modify: `src/lib.rs`
- Test: `tests/cli_init.rs`

- [ ] **Step 1: Write the failing init test**

```rust
#[test]
fn init_creates_project_files() {
    let temp = tempfile::tempdir().unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .args(["init", "--path", temp.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(temp.path().join("electrotest.toml").exists());
    assert!(temp.path().join("features").exists());
    assert!(temp.path().join("steps").exists());
    assert!(temp.path().join("tsconfig.json").exists());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test cli_init init_creates_project_files -- --exact`
Expected: FAIL because the `init` command does not yet create files.

- [ ] **Step 3: Implement template-backed initialization**

```rust
// src/project/templates.rs
pub const DEFAULT_CONFIG: &str = r#"[app]
mode = "launch"
command = "npm"
args = ["run", "start"]

[paths]
features = ["features"]
steps = ["steps"]
artifacts = ".electrotest/artifacts"
"#;
```

```rust
// src/cli/commands/init.rs
pub async fn run(path: &Utf8Path) -> Result<(), crate::Error> {
    tokio::fs::create_dir_all(path.join("features")).await?;
    tokio::fs::create_dir_all(path.join("steps")).await?;
    tokio::fs::write(path.join("electrotest.toml"), templates::DEFAULT_CONFIG).await?;
    tokio::fs::write(path.join("tsconfig.json"), templates::DEFAULT_TSCONFIG).await?;
    Ok(())
}
```

- [ ] **Step 4: Run the init tests**

Run: `cargo test --test cli_init -- --nocapture`
Expected: PASS for the new init behavior and the CLI smoke test.

- [ ] **Step 5: Commit the initialization flow**

```bash
git add src/cli/mod.rs src/cli/commands/init.rs src/project/templates.rs src/lib.rs tests/cli_init.rs
git commit -m "feat: add project initialization command"
```

### Task 3: Add config parsing and validation

**Files:**
- Create: `src/config/mod.rs`
- Create: `src/config/types.rs`
- Create: `src/config/validate.rs`
- Modify: `src/lib.rs`
- Test: `tests/config_loading.rs`

- [ ] **Step 1: Write failing config tests for launch and attach modes**

```rust
#[test]
fn parses_launch_mode_config() {
    let raw = r#"
        [app]
        mode = "launch"
        command = "npm"
        args = ["run", "start"]

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let config = electrotest::config::from_str(raw).unwrap();
    assert_eq!(config.app.mode.as_str(), "launch");
}

#[test]
fn parses_attach_mode_with_endpoint_file() {
    let raw = r#"
        [app]
        mode = "attach"
        endpoint_file = ".electrotest/attach-endpoint.txt"

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let config = electrotest::config::from_str(raw).unwrap();
    assert_eq!(config.app.mode.as_str(), "attach");
    assert_eq!(config.app.endpoint_file.unwrap().as_str(), ".electrotest/attach-endpoint.txt");
}
```

- [ ] **Step 2: Run config tests to verify they fail**

Run: `cargo test --test config_loading -- --nocapture`
Expected: FAIL because `electrotest::config` does not exist.

- [ ] **Step 3: Implement serde-backed config types and validation rules**

```rust
#[derive(Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub paths: PathsConfig,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Launch,
    Attach,
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub mode: AppMode,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub endpoint: Option<String>,
    pub endpoint_file: Option<Utf8PathBuf>,
}
```

```rust
pub fn validate(config: &Config) -> Result<(), ConfigError> {
    match config.app.mode {
        AppMode::Launch if config.app.command.is_none() => Err(ConfigError::MissingLaunchCommand),
        AppMode::Attach if config.app.endpoint.is_none() && config.app.endpoint_file.is_none() => {
            Err(ConfigError::MissingAttachEndpoint)
        }
        AppMode::Attach if config.app.endpoint.is_some() && config.app.endpoint_file.is_some() => {
            Err(ConfigError::ConflictingAttachEndpointSources)
        }
        _ => Ok(()),
    }
}
```

Path resolution rule for this task: resolve all config paths relative to the directory containing the loaded `electrotest.toml` file, not the process working directory.

- [ ] **Step 4: Run the config test suite**

Run: `cargo test --test config_loading -- --nocapture`
Expected: PASS for launch parsing, attach validation, and path handling tests.

- [ ] **Step 5: Commit config support**

```bash
git add src/config/mod.rs src/config/types.rs src/config/validate.rs src/lib.rs tests/config_loading.rs
git commit -m "feat: add electrotest config loading"
```

### Task 4: Implement `electrotest doctor`

**Files:**
- Modify: `src/cli/mod.rs`
- Create: `src/cli/commands/doctor.rs`
- Create: `src/project/bootstrap.rs`
- Modify: `src/config/mod.rs`
- Test: `tests/cli_doctor.rs`

- [ ] **Step 1: Write a failing doctor test around Node detection**

```rust
#[test]
fn doctor_fails_when_node_is_missing() {
    let temp = tempfile::tempdir().unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", "")
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Node.js"));
}

#[test]
fn doctor_reports_missing_feature_and_step_paths() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("electrotest.toml"),
        r#"[app]
mode = "launch"
command = "npm"

[paths]
features = ["missing-features"]
steps = ["missing-steps"]
artifacts = ".electrotest/artifacts"
"#,
    ).unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing-features"));
}
```

- [ ] **Step 2: Run the doctor test to verify it fails**

Run: `cargo test --test cli_doctor doctor_fails_when_node_is_missing -- --exact`
Expected: FAIL because the command is not implemented and no validation checks exist.

- [ ] **Step 3: Implement prerequisite checks and bootstrap stub**

```rust
pub async fn run() -> Result<(), crate::Error> {
    ensure_node_available().await?;
    ensure_supported_node_version().await?;
    ensure_worker_runtime().await?;
    ensure_worker_dependencies().await?;
    let config = crate::config::load_default().await?;
    crate::config::validate_paths(&config)?;
    crate::config::validate_startup(&config)?;
    println!("doctor: ok");
    Ok(())
}
```

```rust
pub async fn ensure_worker_runtime() -> Result<Utf8PathBuf, crate::Error> {
    let cache_dir = directories::ProjectDirs::from("dev", "memo", "electrotest")
        .unwrap()
        .cache_dir()
        .join("worker/v1");
    tokio::fs::create_dir_all(&cache_dir).await?;
    Ok(Utf8PathBuf::from_path_buf(cache_dir).unwrap())
}
```

- [ ] **Step 4: Run the doctor tests**

Run: `cargo test --test cli_doctor -- --nocapture`
Expected: PASS for missing-Node, invalid-Node-version, missing-worker-dependency, invalid startup config, and missing path cases.

- [ ] **Step 5: Commit doctor support**

```bash
git add src/cli/mod.rs src/cli/commands/doctor.rs src/project/bootstrap.rs src/config/mod.rs tests/cli_doctor.rs
git commit -m "feat: add doctor prerequisite checks"
```

### Task 5: Parse and compile Gherkin scenarios

**Files:**
- Create: `src/gherkin/mod.rs`
- Create: `src/gherkin/model.rs`
- Create: `src/gherkin/compile.rs`
- Modify: `src/lib.rs`
- Test: `tests/gherkin_compile.rs`

- [ ] **Step 1: Write a failing compilation test**

```rust
#[test]
fn compiles_feature_into_executable_scenario() {
    let feature = r#"
        Feature: Settings
          Scenario: Open preferences
            Given the Electron app is launched
            When I click on "Settings"
            Then I should see "Preferences"
    "#;

    let compiled = electrotest::gherkin::compile_str(feature).unwrap();
    assert_eq!(compiled.scenarios.len(), 1);
    assert_eq!(compiled.scenarios[0].steps.len(), 3);
}
```

- [ ] **Step 2: Run the Gherkin test to verify it fails**

Run: `cargo test --test gherkin_compile compiles_feature_into_executable_scenario -- --exact`
Expected: FAIL because the Gherkin module does not exist.

- [ ] **Step 3: Implement parsed-to-compiled scenario transformation**

```rust
pub struct CompiledScenario {
    pub feature_name: String,
    pub scenario_name: String,
    pub steps: Vec<CompiledStep>,
}

pub fn compile_str(input: &str) -> Result<CompiledFeature, CompileError> {
    let parsed = gherkin::Feature::parse(input)?;
    compile_feature(parsed)
}
```

- [ ] **Step 4: Run the Gherkin test suite**

Run: `cargo test --test gherkin_compile -- --nocapture`
Expected: PASS for scenario count, step order, and parse error coverage.

- [ ] **Step 5: Commit Gherkin compilation**

```bash
git add src/gherkin/mod.rs src/gherkin/model.rs src/gherkin/compile.rs src/lib.rs tests/gherkin_compile.rs
git commit -m "feat: compile gherkin scenarios"
```

### Task 6: Add the built-in step catalog and selector normalization

**Files:**
- Create: `src/steps/mod.rs`
- Create: `src/steps/builtin.rs`
- Create: `src/steps/registry.rs`
- Create: `src/steps/selector.rs`
- Test: `tests/gherkin_compile.rs`

- [ ] **Step 1: Write a failing test for built-in step resolution**

```rust
#[test]
fn resolves_click_step_to_builtin_handler() {
    let registry = electrotest::steps::Registry::builtin();
    let step = registry.resolve("When I click on \"Settings\"").unwrap();
    assert_eq!(step.action_name(), "click");
}
```

- [ ] **Step 2: Run the step-resolution test to verify it fails**

Run: `cargo test --test gherkin_compile resolves_click_step_to_builtin_handler -- --exact`
Expected: FAIL because the step registry is missing.

- [ ] **Step 3: Implement the V1 step catalog and locator order**

```rust
pub enum Locator {
    Explicit(String),
    TestId(String),
    RoleName { role: String, name: String },
    Text(String),
}

pub fn normalize_target(raw: StepTarget) -> Vec<Locator> {
    vec![
        Locator::Explicit(raw.explicit_selector),
        Locator::TestId(raw.label.clone()),
        Locator::RoleName { role: "button".into(), name: raw.label.clone() },
        Locator::Text(raw.label),
    ]
}
```

- [ ] **Step 4: Run the Gherkin and step tests**

Run: `cargo test --test gherkin_compile -- --nocapture`
Expected: PASS for built-in step matching and selector normalization tests.

- [ ] **Step 5: Commit the built-in step system**

```bash
git add src/steps/mod.rs src/steps/builtin.rs src/steps/registry.rs src/steps/selector.rs tests/gherkin_compile.rs
git commit -m "feat: add builtin electrotest steps"
```

### Task 7: Add the worker protocol and process wrapper

**Files:**
- Create: `src/engine/mod.rs`
- Create: `src/engine/protocol.rs`
- Create: `src/engine/process.rs`
- Create: `src/engine/playwright.rs`
- Test: `tests/worker_protocol.rs`

- [ ] **Step 1: Write a failing protocol round-trip test**

```rust
#[tokio::test]
async fn serializes_and_reads_worker_response() {
    let request = electrotest::engine::protocol::Request::Ping;
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"type\":\"ping\""));
}
```

- [ ] **Step 2: Run the worker protocol test to verify it fails**

Run: `cargo test --test worker_protocol serializes_and_reads_worker_response -- --exact`
Expected: FAIL because the protocol module does not exist.

- [ ] **Step 3: Implement JSON protocol types and worker process wrapper**

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Ping,
    LaunchApp { command: String, args: Vec<String> },
    AttachApp { endpoint: String },
    Click { window_id: String, locator: Vec<LocatorPayload> },
    Screenshot { window_id: String },
}
```

```rust
pub struct WorkerProcess {
    child: tokio::process::Child,
    stdin: ChildStdin,
    stdout: FramedRead<ChildStdout, LinesCodec>,
}
```

- [ ] **Step 4: Run the worker protocol tests**

Run: `cargo test --test worker_protocol -- --nocapture`
Expected: PASS for serialization, attach and launch request coverage, startup, and malformed-response tests.

- [ ] **Step 5: Commit worker protocol support**

```bash
git add src/engine/mod.rs src/engine/protocol.rs src/engine/process.rs src/engine/playwright.rs tests/worker_protocol.rs
git commit -m "feat: add playwright worker protocol"
```

### Task 8: Add the Node worker runtime and bootstrap installation

**Files:**
- Create: `runtime/worker/package.json`
- Create: `runtime/worker/tsconfig.json`
- Create: `runtime/worker/src/index.ts`
- Create: `runtime/worker/src/protocol.ts`
- Create: `runtime/worker/src/engine.ts`
- Create: `runtime/worker/scripts/build.mjs`
- Modify: `src/project/bootstrap.rs`
- Test: `tests/worker_protocol.rs`

- [ ] **Step 1: Write a failing bootstrap test for runtime materialization**

```rust
#[tokio::test]
async fn bootstraps_worker_runtime_into_cache() {
    let cache = tempfile::tempdir().unwrap();
    let runtime = electrotest::project::bootstrap::materialize_runtime(cache.path()).await.unwrap();
    assert!(runtime.join("index.js").exists());
}
```

- [ ] **Step 2: Run the bootstrap test to verify it fails**

Run: `cargo test --test worker_protocol bootstraps_worker_runtime_into_cache -- --exact`
Expected: FAIL because runtime assets are not copied, built, or dependency-installed in the cache.

- [ ] **Step 3: Add the worker sources, build script, and cache bootstrap implementation**

```json
{
  "name": "electrotest-worker",
  "private": true,
  "type": "module",
  "scripts": {
    "build": "node ./scripts/build.mjs"
  },
  "dependencies": {
    "playwright": "^1.52.0"
  },
  "devDependencies": {
    "typescript": "^5.8.0"
  }
}
```

```js
// runtime/worker/scripts/build.mjs
import { execFileSync } from "node:child_process";

execFileSync(process.execPath, ["./node_modules/typescript/bin/tsc", "-p", "tsconfig.json"], {
  stdio: "inherit",
});
```

```ts
// runtime/worker/src/index.ts
process.stdin.setEncoding("utf8");
for await (const line of readLines(process.stdin)) {
  const request = JSON.parse(line);
  const response = await handleRequest(request);
  process.stdout.write(JSON.stringify(response) + "\n");
}
```

```rust
pub async fn materialize_runtime(cache_root: &Path) -> Result<Utf8PathBuf, crate::Error> {
    copy_embedded_runtime(cache_root).await?;
    run_npm_install(cache_root).await?;
    run_worker_build(cache_root).await?;
    Ok(cache_root.join("dist"))
}
```

- [ ] **Step 4: Run the worker protocol tests again**

Run: `cargo test --test worker_protocol -- --nocapture`
Expected: PASS for runtime bootstrap, dependency installation checks, build output checks, and process startup tests.

- [ ] **Step 5: Commit the Node runtime bundle**

```bash
git add runtime/worker src/project/bootstrap.rs tests/worker_protocol.rs
git commit -m "feat: bundle playwright worker runtime"
```

### Task 9: Add shared fixture harness for integration tests

**Files:**
- Create: `tests/support/mod.rs`
- Create: `tests/fixtures/electron-app/package.json`
- Create: `tests/fixtures/electron-app/main.js`
- Create: `tests/fixtures/features/basic-launch.feature`
- Create: `tests/fixtures/features/multi-window.feature`
- Create: `tests/fixtures/features/custom-step.feature`
- Create: `tests/fixtures/features/failing-assertion.feature`
- Create: `tests/fixtures/steps/sample.steps.ts`
- Test: `tests/e2e_fixture.rs`

- [ ] **Step 1: Write the failing shared-fixture smoke test**

```rust
#[tokio::test]
async fn fixture_harness_returns_paths_for_test_assets() {
    let fixture = support::fixture_project().await;
    assert!(fixture.root.join("electron-app/package.json").exists());
    assert!(fixture.root.join("features/basic-launch.feature").exists());
}
```

- [ ] **Step 2: Run the fixture smoke test to verify it fails**

Run: `cargo test --test e2e_fixture fixture_harness_returns_paths_for_test_assets -- --exact --nocapture`
Expected: FAIL because the fixture harness and files do not exist.

- [ ] **Step 3: Add the Electron fixture app, fixture features, and shared test helper**

```json
{
  "name": "electrotest-fixture-app",
  "private": true,
  "main": "main.js",
  "scripts": {
    "start": "electron ."
  },
  "devDependencies": {
    "electron": "^35.0.0"
  }
}
```

```rust
// tests/support/mod.rs
pub async fn fixture_project() -> FixtureProject {
    FixtureProject::from_repo_paths("tests/fixtures").await
}

pub struct FixtureRun {
    pub status: std::process::ExitStatus,
    pub stdout: String,
    pub artifact_dir: std::path::PathBuf,
}

pub async fn run_fixture(feature_name: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    run_electrotest_fixture(feature_name, None).await
}

pub async fn run_attach_fixture(feature_name: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    run_electrotest_fixture(feature_name, Some("tests/fixtures/attach/electrotest.toml")).await
}
```

- [ ] **Step 4: Run the fixture smoke test again**

Run: `cargo test --test e2e_fixture fixture_harness_returns_paths_for_test_assets -- --exact --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit the shared fixture harness**

```bash
git add tests/support/mod.rs tests/e2e_fixture.rs tests/fixtures
git commit -m "test: add electrotest fixture harness"
```

### Task 10: Implement custom `JS/TS` step loading

**Files:**
- Create: `runtime/worker/src/steps.ts`
- Create: `runtime/worker/src/sdk.ts`
- Modify: `src/steps/registry.rs`
- Modify: `src/engine/playwright.rs`
- Test: `tests/worker_protocol.rs`
- Test: `tests/e2e_fixture.rs`

- [ ] **Step 1: Write a failing integration test for TypeScript step execution**

```rust
#[tokio::test]
async fn executes_custom_typescript_step() {
    let result = support::run_fixture("custom-step.feature").await;
    assert!(result.stdout.contains("custom step executed"));
}
```

- [ ] **Step 2: Run the custom-step test to verify it fails**

Run: `cargo test --test e2e_fixture executes_custom_typescript_step -- --exact --nocapture`
Expected: FAIL because custom step modules are not loaded.

- [ ] **Step 3: Implement the bounded SDK and TS transpilation flow**

```ts
// runtime/worker/src/sdk.ts
export function defineStep(pattern: RegExp | string, handler: StepHandler): RegisteredStep {
  return { pattern, handler };
}
```

```ts
// runtime/worker/src/steps.ts
export async function loadStepModules(stepPaths: string[]) {
  const compiled = await transpileTypescriptIfNeeded(stepPaths);
  return Promise.all(compiled.map((file) => import(pathToFileURL(file).href)));
}
```

- [ ] **Step 4: Run the protocol tests including custom steps**

Run: `cargo test --test worker_protocol --test e2e_fixture executes_custom_typescript_step -- --nocapture`
Expected: PASS for JS loading, TS transpilation, pattern registration, and fixture-backed custom step execution.

- [ ] **Step 5: Commit custom step support**

```bash
git add runtime/worker/src/steps.ts runtime/worker/src/sdk.ts src/steps/registry.rs src/engine/playwright.rs tests/worker_protocol.rs tests/fixtures/steps/sample.steps.ts
git commit -m "feat: load custom js and ts steps"
```

### Task 11: Implement the scenario runner, artifacts, and error classification

**Files:**
- Create: `src/runner/mod.rs`
- Create: `src/runner/context.rs`
- Create: `src/runner/artifacts.rs`
- Create: `src/runner/errors.rs`
- Create: `src/runner/execute.rs`
- Modify: `src/cli/commands/test.rs`
- Test: `tests/runner_errors.rs`
- Test: `tests/e2e_fixture.rs`

- [ ] **Step 1: Write a failing runner test for artifact capture on step failure**

```rust
#[tokio::test]
async fn stores_screenshot_and_trace_when_step_fails() {
    let result = support::run_fixture("failing-assertion.feature").await;
    assert!(result.artifact_dir.join("failure.png").exists());
    assert!(result.artifact_dir.join("trace.zip").exists());
}

#[tokio::test]
async fn test_command_returns_non_zero_on_failure() {
    let result = support::run_fixture("failing-assertion.feature").await;
    assert!(!result.status.success());
    assert!(result.stdout.contains("1 scenario failed"));
}
```

- [ ] **Step 2: Run the runner test to verify it fails**

Run: `cargo test --test runner_errors stores_screenshot_and_trace_when_step_fails -- --exact`
Expected: FAIL because the runner is not implemented.

- [ ] **Step 3: Implement execution context, error mapping, and artifact writes**

```rust
pub enum RunError {
    Config(String),
    MissingStep(String),
    ElementNotFound(String),
    Timeout(String),
    Crash(String),
    Assertion(String),
}
```

```rust
pub async fn execute(run: RunRequest) -> Result<RunSummary, RunError> {
    for scenario in run.scenarios {
        execute_scenario(&run.engine, &scenario).await?;
    }
    Ok(RunSummary::success())
}
```

```rust
pub async fn run(args: TestArgs) -> Result<(), crate::Error> {
    let summary = runner::execute(args.into_request()).await?;
    println!("{} scenario passed, {} failed", summary.passed, summary.failed);
    if summary.failed > 0 {
        return Err(crate::Error::TestFailures(summary.failed));
    }
    Ok(())
}
```

- [ ] **Step 4: Run the runner tests**

Run: `cargo test --test runner_errors --test e2e_fixture test_command_returns_non_zero_on_failure -- --nocapture`
Expected: PASS for error classification, artifact path creation, failure-summary behavior, and non-zero exit code coverage.

- [ ] **Step 5: Commit the runner core**

```bash
git add src/runner/mod.rs src/runner/context.rs src/runner/artifacts.rs src/runner/errors.rs src/runner/execute.rs src/cli/commands/test.rs tests/runner_errors.rs
git commit -m "feat: add electrotest scenario runner"
```

### Task 12: Implement `electrotest list` and complete command dispatch

**Files:**
- Modify: `src/cli/mod.rs`
- Create: `src/cli/commands/list.rs`
- Modify: `src/cli/commands/test.rs`
- Test: `tests/gherkin_compile.rs`

- [ ] **Step 1: Write a failing test for scenario listing**

```rust
#[test]
fn list_prints_feature_and_scenario_names() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("sample.feature"),
        "Feature: Settings\n  Scenario: Open preferences\n    Given the Electron app is launched\n",
    ).unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .args(["list", "--features", temp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("Open preferences"));
}
```

- [ ] **Step 2: Run the listing test to verify it fails**

Run: `cargo test --test gherkin_compile list_prints_feature_and_scenario_names -- --exact`
Expected: FAIL because the `list` command is not implemented.

- [ ] **Step 3: Implement listing and full command dispatch**

```rust
match cli.command.unwrap_or(Commands::Test(TestArgs::default())) {
    Commands::Init(args) => commands::init::run(&args.path).await,
    Commands::Doctor(args) => commands::doctor::run(args).await,
    Commands::List(args) => commands::list::run(args).await,
    Commands::Test(args) => commands::test::run(args).await,
}
```

- [ ] **Step 4: Run command-level tests**

Run: `cargo test --test cli_init --test cli_doctor --test gherkin_compile -- --nocapture`
Expected: PASS for help, init, doctor, Gherkin compile, built-in step resolution, and list command coverage.

- [ ] **Step 5: Commit command completion**

```bash
git add src/cli/mod.rs src/cli/commands/list.rs src/cli/commands/test.rs tests/gherkin_compile.rs
git commit -m "feat: add feature listing command"
```

### Task 13: Add end-to-end fixture coverage for launch, attach, windows, and custom steps

**Files:**
- Modify: `tests/e2e_fixture.rs`
- Modify: `tests/support/mod.rs`
- Modify: `tests/fixtures/features/multi-window.feature`
- Create: `tests/fixtures/features/attach-mode.feature`
- Create: `tests/fixtures/features/missing-window.feature`
- Create: `tests/fixtures/features/ambiguous-window.feature`
- Create: `tests/fixtures/attach/electrotest.toml`
- Create: `tests/fixtures/attach/start-attached-session.mjs`
- Create: `tests/fixtures/attach/package.json`
- Modify: `src/cli/commands/test.rs`
- Modify: `src/engine/protocol.rs`
- Modify: `src/engine/playwright.rs`

- [ ] **Step 1: Write the failing end-to-end fixture test**

```rust
#[tokio::test]
async fn runs_feature_against_fixture_electron_app() {
    let result = support::run_fixture("basic-launch.feature").await;
    assert!(result.status.success());
    assert!(result.stdout.contains("1 scenario passed"));
}

#[tokio::test]
async fn attach_mode_can_run_against_existing_fixture_app() {
    let result = support::run_attach_fixture("attach-mode.feature").await;
    assert!(result.status.success());
}

#[tokio::test]
async fn switches_window_by_title_in_multi_window_scenario() {
    let result = support::run_fixture("multi-window.feature").await;
    assert!(result.status.success());
    assert!(result.stdout.contains("Switched to window: Preferences"));
}

#[tokio::test]
async fn switches_window_by_index_in_multi_window_scenario() {
    let result = support::run_fixture("multi-window.feature").await;
    assert!(result.status.success());
    assert!(result.stdout.contains("Switched to window index 1"));
}

#[tokio::test]
async fn reports_clear_error_when_window_target_is_missing() {
    let result = support::run_fixture("missing-window.feature").await;
    assert!(!result.status.success());
    assert!(result.stdout.contains("window target not found"));
}

#[tokio::test]
async fn reports_clear_error_when_window_target_is_ambiguous() {
    let result = support::run_fixture("ambiguous-window.feature").await;
    assert!(!result.status.success());
    assert!(result.stdout.contains("window target is ambiguous"));
}
```

- [ ] **Step 2: Run the end-to-end test to verify it fails**

Run: `cargo test --test e2e_fixture runs_feature_against_fixture_electron_app -- --exact --nocapture`
Expected: FAIL because full launch, attach, and multi-window wiring are not complete yet.

- [ ] **Step 3: Wire `electrotest test` through the full stack for launch and attach flows**

```rust
Request::AttachApp { endpoint } => engine.attach(endpoint).await,
Request::LaunchApp { command, args } => engine.launch(command, args).await,
```

```toml
# tests/fixtures/attach/electrotest.toml
[app]
mode = "attach"
endpoint_file = ".electrotest/attach-endpoint.txt"

[paths]
features = ["../features"]
steps = ["../steps"]
artifacts = ".electrotest/artifacts"
```

```json
// tests/fixtures/attach/package.json
{
  "private": true,
  "type": "module",
  "dependencies": {
    "wait-on": "^8.0.0"
  }
}
```

```js
// tests/fixtures/attach/start-attached-session.mjs
import { spawn } from "node:child_process";
import fs from "node:fs";

const port = 9333;
const endpointFile = process.argv[2];
const child = spawn("./tests/fixtures/electron-app/node_modules/.bin/electron", ["./tests/fixtures/electron-app", `--remote-debugging-port=${port}`], {
  stdio: "inherit",
});

fs.writeFileSync(endpointFile, `http://127.0.0.1:${port}`);
process.on("exit", () => child.kill("SIGTERM"));
```

Implementation requirements for this step:

- update the fixture helper so `run_attach_fixture(...)` installs `tests/fixtures/attach/package.json` if needed, starts `tests/fixtures/attach/start-attached-session.mjs`, waits for `.electrotest/attach-endpoint.txt`, then runs `electrotest test` with `tests/fixtures/attach/electrotest.toml`
- update the fixture helper so `ensure_fixture_dependencies()` installs `tests/fixtures/electron-app/package.json` dependencies before any launch or attach fixture run
- update the attach fixture helper to wait until the CDP endpoint accepts connections after the endpoint file appears, to avoid flaky startup timing
- update the worker protocol and engine to accept a CDP endpoint and connect through `playwright.chromium.connectOverCDP(...)`; attach mode is intentionally limited to renderer-driven automation in V1
- implement default active-window behavior on launch
- implement window switching by title and index
- surface explicit errors for ambiguous and missing window matches

- [ ] **Step 4: Run the full verification suite**

Run: `cargo test -- --nocapture`
Expected: PASS across unit, integration, and fixture tests, including launch, attach, multi-window switching, missing-window error coverage, custom-step execution, and failure-artifact coverage. If Electron-based tests are environment-sensitive, document the exact env vars or CI guard needed inside the test helper and re-run until stable.

- [ ] **Step 5: Commit the end-to-end fixture coverage**

```bash
git add tests/e2e_fixture.rs tests/support/mod.rs tests/fixtures src/cli/commands/test.rs src/engine/protocol.rs src/engine/playwright.rs
git commit -m "test: add fixture coverage for electrotest e2e flow"
```

## Final Verification

- [ ] Run: `cargo fmt --check`
Expected: PASS.

- [ ] Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS.

- [ ] Run: `cargo test -- --nocapture`
Expected: PASS.

- [ ] Run: `cargo run -- init --path /tmp/electrotest-smoke`
Expected: creates `electrotest.toml`, `features/`, `steps/`, and `tsconfig.json`.

- [ ] Run: `cargo run -- doctor`
Expected: reports Node/runtime readiness or a precise actionable failure.
