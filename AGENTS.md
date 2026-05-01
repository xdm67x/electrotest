# AGENTS.md

Compact guidance for AI coding agents (OpenCode, Claude, Cursor, Copilot, etc.) working on the electrotest codebase.

## Build and Test

```bash
# Standard Rust workflow — no custom scripts
cargo build
cargo test
cargo test test_name       # run a single parser unit test
cargo run -- --pid 12345 --features ./examples/features/test.feature --output-dir ./output
```

## Toolchain

- Rust is pinned to **1.94** in `rust-toolchain.toml`.
- Cross-compilation targets are declared there (`aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-musl`).
- CI release builds for all three targets on tag push (`v*`).

## Architecture

Electrotest is a CLI that runs Gherkin `.feature` files against an Electron process via the Chrome DevTools Protocol (CDP).

**Entry point**: `src/main.rs` creates a tokio runtime and calls `cli::run()`.

**Layer map** (non-obvious from filenames):
- `src/cli/args.rs` — clap CLI parsing.
- `src/cli/parser.rs` — regex-based Gherkin parser. Contains the only unit tests in the repo.
- `src/cli/runner.rs` — orchestrates scenario execution. **And/But keywords resolve to the previous non-And/But keyword** here.
- `src/cli/context.rs` — shared mutable state (CDP client, output dir, window size).
- `src/cli/steps/` — trait-based step handlers (`StepHandler`).
- `src/cdp/client.rs` — WebSocket CDP client using `tokio-tungstenite`. Auto-increments message IDs; correlates responses via `HashMap<MessageId, oneshot::Sender>`.

**PID → Port Discovery**: `sysinfo` reads the Electron process command line to extract `--remote-debugging-port`, then hits `http://127.0.0.1:{port}/json/list` to find the WebSocket debugger URL before connecting.

## Adding a New Gherkin Step

1. Implement `StepHandler` (trait in `src/cli/steps/mod.rs`) in the appropriate `src/cli/steps/*.rs` file.
2. Implement `can_handle()` with regex-like matching on step text.
3. Implement `execute()` using `ctx.cdp_client.evaluate()` or other CDP helpers.
4. **Manually register** the handler in `StepRegistry::new()` in `src/cli/steps/mod.rs`.

## Testing Constraints

- Only unit tests exist (in `src/cli/parser.rs`).
- **No integration tests** — running the full tool requires a live Electron process with `--remote-debugging-port` enabled.
- CI (`/.github/workflows/release.yml`) only builds releases; it does not run `cargo test`.

## Release Workflow

Release automation is documented as a skill at `.agents/skills/electrotest-release/SKILL.md`. The full sequence covers version bumping, changelog generation, git tagging, CI monitoring, and Homebrew tap updates.

A helper script for generating the Homebrew formula lives at:
`.agents/skills/electrotest-release/scripts/update-homebrew.sh`

## Style and Workflow

- Use `cargo fmt` and `cargo clippy` — no custom linter config.
- Keep `AGENTS.md` in sync if you change architecture or build steps.
